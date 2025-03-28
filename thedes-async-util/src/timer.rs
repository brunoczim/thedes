use std::{
    mem,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    time::Duration,
};

use tokio::time::{self, Instant, Interval};

type Id = usize;

#[derive(Debug)]
enum Descriptor {
    Vaccant,
    NotWaiting,
    Waiting(Waker),
}

#[derive(Debug)]
struct State {
    interval: Interval,
    descriptors: Vec<Descriptor>,
    participants: usize,
    waiting: usize,
    last_tick: Instant,
}

impl State {
    pub fn new(interval: Interval, start: Instant) -> Self {
        Self {
            interval,
            descriptors: Vec::new(),
            participants: 0,
            waiting: 0,
            last_tick: start,
        }
    }

    pub fn new_participant(&mut self) -> Id {
        self.participants += 1;
        for (id, descriptor) in self.descriptors.iter_mut().enumerate() {
            if matches!(descriptor, Descriptor::Vaccant) {
                *descriptor = Descriptor::NotWaiting;
                return id;
            }
        }
        let id = self.descriptors.len();
        self.descriptors.push(Descriptor::NotWaiting);
        id
    }

    pub fn drop_participant(&mut self, id: Id) {
        let descriptor = &mut self.descriptors[id];
        let waiting = match descriptor {
            Descriptor::Vaccant => {
                debug_assert!(false, "cannot drop vaccant participant");
                false
            },
            Descriptor::NotWaiting => false,
            Descriptor::Waiting(_) => true,
        };
        *descriptor = Descriptor::Vaccant;
        if waiting {
            self.cancel_one_waiting();
        }
        self.participants -= 1;
        while let Some(Descriptor::Vaccant) = self.descriptors.last() {
            self.descriptors.pop();
        }
    }

    pub fn poll_tick(
        &mut self,
        cx: &mut Context<'_>,
        id: Id,
        last_known_tick: Instant,
    ) -> Poll<Instant> {
        let descriptor =
            mem::replace(&mut self.descriptors[id], Descriptor::NotWaiting);

        match descriptor {
            Descriptor::Vaccant => {
                debug_assert!(false, "cannot poll on vaccant descriptor");
                Poll::Pending
            },
            Descriptor::NotWaiting => {
                if last_known_tick >= self.last_tick {
                    self.waiting += 1;
                    self.descriptors[id] =
                        Descriptor::Waiting(cx.waker().clone());
                    self.poll_interval(cx)
                } else {
                    Poll::Ready(self.last_tick)
                }
            },
            Descriptor::Waiting(_) => {
                if last_known_tick >= self.last_tick {
                    self.descriptors[id] =
                        Descriptor::Waiting(cx.waker().clone());
                    self.poll_interval(cx)
                } else {
                    debug_assert!(
                        false,
                        "last known tick should be up to date"
                    );
                    Poll::Ready(self.last_tick)
                }
            },
        }
    }

    pub fn cancel_tick(&mut self, id: Id) -> bool {
        let descriptor = &mut self.descriptors[id];
        debug_assert_eq!(
            false,
            matches!(descriptor, Descriptor::Vaccant),
            "cannot cancel on vaccant descriptor",
        );
        if matches!(descriptor, Descriptor::Waiting(_)) {
            *descriptor = Descriptor::NotWaiting;
            self.cancel_one_waiting();
            true
        } else {
            false
        }
    }

    fn poll_interval(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        if self.participants <= self.waiting {
            debug_assert_eq!(self.participants, self.waiting);
            let poll = self.interval.poll_tick(cx);
            if let Poll::Ready(instant) = poll {
                self.last_tick = instant;
                self.waiting = 0;
                for descriptor in &mut self.descriptors {
                    if let Descriptor::Waiting(waker) =
                        mem::replace(descriptor, Descriptor::NotWaiting)
                    {
                        waker.wake();
                    }
                }
            }
            poll
        } else {
            Poll::Pending
        }
    }

    fn cancel_one_waiting(&mut self) {
        if self.participants <= self.waiting {
            debug_assert_eq!(self.participants, self.waiting);
            self.waiting -= 1;
            for descriptor in &mut self.descriptors {
                match mem::replace(descriptor, Descriptor::NotWaiting) {
                    Descriptor::Waiting(waker) => {
                        waker.wake();
                        self.waiting -= 1;
                        break;
                    },
                    value => *descriptor = value,
                }
            }
        }
    }
}

#[derive(Debug)]
struct Shared {
    period: Duration,
    state: Mutex<State>,
}

impl Shared {
    pub fn with_state<F, T>(&self, scope: F) -> T
    where
        F: FnOnce(&mut State) -> T,
    {
        let mut state = self.state.lock().expect("poisoned lock");
        scope(&mut state)
    }
}

#[derive(Debug, Clone)]
pub struct Timer {
    shared: Arc<Shared>,
}

impl Timer {
    pub fn new(period: Duration) -> Self {
        let start = Instant::now();
        let interval = time::interval(period);
        let state = Mutex::new(State::new(interval, start));
        let shared = Shared { period, state };
        Self { shared: Arc::new(shared) }
    }

    fn from_participant(participant: &TickParticipant) -> Self {
        Self { shared: participant.shared.clone() }
    }

    pub fn new_participant(&self) -> TickParticipant {
        let last_known_tick = self.shared.with_state(|state| state.last_tick);
        TickParticipant::new(&self.shared, last_known_tick)
    }
}

#[derive(Debug)]
pub struct TickParticipant {
    id: Id,
    last_known_tick: Instant,
    shared: Arc<Shared>,
}

impl TickParticipant {
    fn new(shared: &Arc<Shared>, last_known_tick: Instant) -> Self {
        let id = shared.with_state(|state| state.new_participant());
        Self { id, last_known_tick, shared: shared.clone() }
    }

    pub fn tick(&mut self) -> Tick<'_> {
        Tick { timer: self }
    }

    pub fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        let poll = self.shared.with_state(|state| {
            state.poll_tick(cx, self.id, self.last_known_tick)
        });
        if let Poll::Ready(instant) = poll {
            self.last_known_tick = instant;
        }
        poll
    }

    pub fn cancel_tick(&mut self) -> bool {
        self.shared.with_state(|state| state.cancel_tick(self.id))
    }

    pub fn period(&self) -> Duration {
        self.shared.period
    }

    pub fn last_tick(&self) -> Instant {
        self.shared.with_state(|state| state.last_tick)
    }

    pub fn elapsed(&self) -> Duration {
        self.last_known_tick.elapsed()
    }

    pub fn time_left(&self) -> Duration {
        self.period().saturating_sub(self.elapsed())
    }

    pub fn timer(&self) -> Timer {
        Timer::from_participant(self)
    }
}

impl Clone for TickParticipant {
    fn clone(&self) -> Self {
        Self::new(&self.shared, self.last_known_tick)
    }
}

impl Drop for TickParticipant {
    fn drop(&mut self) {
        self.shared.with_state(|state| state.drop_participant(self.id))
    }
}

#[derive(Debug)]
pub struct Tick<'a> {
    timer: &'a mut TickParticipant,
}

impl Future for Tick<'_> {
    type Output = Instant;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        self.timer.poll_tick(cx)
    }
}

impl Drop for Tick<'_> {
    fn drop(&mut self) {
        self.timer.cancel_tick();
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::{task::JoinSet, time};

    use crate::timer::Timer;

    #[tokio::test]
    async fn sync_once() {
        let mut join_set = JoinSet::new();
        let timer = Timer::new(Duration::from_micros(100));
        let mut participant = timer.new_participant();
        let timers = (0 .. 16).map(|_| participant.clone()).collect::<Vec<_>>();
        for mut participant in timers {
            join_set.spawn(async move { participant.tick().await });
        }
        let answer = participant.tick().await;
        while let Some(alternative) = join_set.join_next().await {
            assert_eq!(answer, alternative.unwrap());
        }
    }

    #[tokio::test]
    async fn sync_twice() {
        let mut join_set = JoinSet::new();
        let timer = Timer::new(Duration::from_micros(100));
        let mut participant = timer.new_participant();
        let timers = (0 .. 16).map(|_| participant.clone()).collect::<Vec<_>>();
        for mut participant in timers {
            join_set.spawn(async move {
                let first = participant.tick().await;
                let second = participant.tick().await;
                (first, second)
            });
        }
        let first_answer = participant.tick().await;
        let second_answer = participant.tick().await;
        while let Some(alternative) = join_set.join_next().await {
            let (first_alternative, second_alternative) = alternative.unwrap();
            assert_eq!(first_answer, first_alternative);
            assert_eq!(second_answer, second_alternative);
        }
    }

    #[tokio::test]
    async fn sync_twice_with_novices() {
        let mut join_set = JoinSet::new();
        let timer = Timer::new(Duration::from_micros(100));
        let mut participant = timer.new_participant();
        let timers = (0 .. 16).map(|_| participant.clone()).collect::<Vec<_>>();
        for mut participant in timers {
            join_set.spawn(async move {
                let first = participant.tick().await;
                let second = participant.tick().await;
                (Some(first), second)
            });
        }
        let first_answer = participant.tick().await;
        let timers = (0 .. 7).map(|_| participant.clone()).collect::<Vec<_>>();
        for mut participant in timers {
            join_set.spawn(async move {
                let second = participant.tick().await;
                (None, second)
            });
        }
        let second_answer = participant.tick().await;

        let mut has_first = 0;
        while let Some(alternative) = join_set.join_next().await {
            let (maybe_first_alternative, second_alternative) =
                alternative.unwrap();
            if let Some(first_alternative) = maybe_first_alternative {
                assert_eq!(first_answer, first_alternative);
                has_first += 1;
            }
            assert_eq!(second_answer, second_alternative);
        }
        assert_ne!(has_first, 0);
    }

    #[tokio::test]
    async fn sync_thrice_with_novices_and_leavos() {
        let mut join_set = JoinSet::new();
        let timer = Timer::new(Duration::from_micros(100));
        let mut participant = timer.new_participant();
        let timers = (0 .. 16).map(|_| participant.clone()).collect::<Vec<_>>();
        for (i, mut participant) in timers.into_iter().enumerate() {
            join_set.spawn(async move {
                let first = participant.tick().await;
                let second = participant.tick().await;
                let third =
                    if i < 4 { None } else { Some(participant.tick().await) };
                (Some(first), second, third)
            });
        }
        let first_answer = participant.tick().await;
        let timers = (0 .. 7).map(|_| participant.clone()).collect::<Vec<_>>();
        for mut participant in timers {
            join_set.spawn(async move {
                let second = participant.tick().await;
                let third = participant.tick().await;
                (None, second, Some(third))
            });
        }
        let second_answer = participant.tick().await;
        let third_answer = participant.tick().await;

        let mut has_first = 0;
        let mut has_third = 0;
        while let Some(alternative) = join_set.join_next().await {
            let (
                maybe_first_alternative,
                second_alternative,
                maybe_third_alternative,
            ) = alternative.unwrap();
            if let Some(first_alternative) = maybe_first_alternative {
                assert_eq!(first_answer, first_alternative);
                has_first += 1;
            }
            assert_eq!(second_answer, second_alternative);
            if let Some(third_alternative) = maybe_third_alternative {
                assert_eq!(third_answer, third_alternative);
                has_third += 1;
            }
        }
        assert_ne!(has_first, 0);
        assert_ne!(has_third, 0);
    }

    #[tokio::test]
    async fn sync_thrice_with_novices_and_leavos_mixed() {
        let mut join_set = JoinSet::new();
        let timer = Timer::new(Duration::from_micros(100));
        let mut participant = timer.new_participant();
        let timers = (0 .. 16).map(|_| participant.clone()).collect::<Vec<_>>();
        for (i, mut participant) in timers.into_iter().enumerate() {
            join_set.spawn(async move {
                let first = participant.tick().await;
                let (second, third) = if i < 13 {
                    let second = participant.tick().await;
                    let third = if i < 4 {
                        None
                    } else {
                        Some(participant.tick().await)
                    };
                    (Some(second), third)
                } else {
                    (None, None)
                };
                (Some(first), second, third)
            });
        }
        let first_answer = participant.tick().await;
        let timers = (0 .. 7).map(|_| participant.clone()).collect::<Vec<_>>();
        for mut participant in timers {
            join_set.spawn(async move {
                let second = participant.tick().await;
                let third = participant.tick().await;
                (None, Some(second), Some(third))
            });
        }
        let second_answer = participant.tick().await;
        let third_answer = participant.tick().await;

        let mut has_first = 0;
        let mut has_second = 0;
        let mut has_third = 0;
        while let Some(alternative) = join_set.join_next().await {
            let (
                maybe_first_alternative,
                maybe_second_alternative,
                maybe_third_alternative,
            ) = alternative.unwrap();
            if let Some(first_alternative) = maybe_first_alternative {
                assert_eq!(first_answer, first_alternative);
                has_first += 1;
            }
            if let Some(second_alternative) = maybe_second_alternative {
                assert_eq!(second_answer, second_alternative);
                has_second += 1;
            }
            if let Some(third_alternative) = maybe_third_alternative {
                assert_eq!(third_answer, third_alternative);
                has_third += 1;
            }
        }
        assert_ne!(has_first, 0);
        assert_ne!(has_second, 0);
        assert_ne!(has_third, 0);
    }

    #[tokio::test]
    async fn sync_four_times_with_novices_and_leavos_and_novices_again() {
        let mut join_set = JoinSet::new();
        let timer = Timer::new(Duration::from_micros(100));
        let mut participant = timer.new_participant();
        let timers = (0 .. 16).map(|_| participant.clone()).collect::<Vec<_>>();
        for (i, mut participant) in timers.into_iter().enumerate() {
            join_set.spawn(async move {
                let first = participant.tick().await;
                let second = participant.tick().await;
                let third =
                    if i < 9 { None } else { Some(participant.tick().await) };
                (Some(first), Some(second), third, None)
            });
        }
        let first_answer = participant.tick().await;
        let timers = (0 .. 7).map(|_| participant.clone()).collect::<Vec<_>>();
        for mut participant in timers {
            join_set.spawn(async move {
                let second = participant.tick().await;
                let third = participant.tick().await;
                let fourth = participant.tick().await;
                (None, Some(second), Some(third), Some(fourth))
            });
        }
        let second_answer = participant.tick().await;
        let third_answer = participant.tick().await;
        let timers = (0 .. 11).map(|_| participant.clone()).collect::<Vec<_>>();
        for mut participant in timers {
            join_set.spawn(async move {
                let fourth = participant.tick().await;
                (None, None, None, Some(fourth))
            });
        }
        let fourth_answer = participant.tick().await;

        let mut has_first = 0;
        let mut has_second = 0;
        let mut has_third = 0;
        let mut has_fourth = 0;
        while let Some(alternative) = join_set.join_next().await {
            let (
                maybe_first_alternative,
                maybe_second_alternative,
                maybe_third_alternative,
                maybe_fourth_alternative,
            ) = alternative.unwrap();
            if let Some(first_alternative) = maybe_first_alternative {
                assert_eq!(first_answer, first_alternative);
                has_first += 1;
            }
            if let Some(second_alternative) = maybe_second_alternative {
                assert_eq!(second_answer, second_alternative);
                has_second += 1;
            }
            if let Some(third_alternative) = maybe_third_alternative {
                assert_eq!(third_answer, third_alternative);
                has_third += 1;
            }
            if let Some(fourth_alternative) = maybe_fourth_alternative {
                assert_eq!(fourth_answer, fourth_alternative);
                has_fourth += 1;
            }
        }
        assert_ne!(has_first, 0);
        assert_ne!(has_second, 0);
        assert_ne!(has_third, 0);
        assert_ne!(has_fourth, 0);
    }

    #[tokio::test]
    async fn sync_twice_only_one_left() {
        let mut join_set = JoinSet::new();
        let timer = Timer::new(Duration::from_micros(100));
        let mut participant = timer.new_participant();
        let timers = (0 .. 16).map(|_| participant.clone()).collect::<Vec<_>>();
        for mut participant in timers {
            join_set.spawn(async move { participant.tick().await });
        }
        let first_answer = participant.tick().await;
        let _second_answer = participant.tick().await;
        while let Some(alternative) = join_set.join_next().await {
            assert_eq!(first_answer, alternative.unwrap());
        }
    }

    #[tokio::test]
    async fn sync_with_cancel() {
        let mut join_set = JoinSet::new();
        let timer = Timer::new(Duration::from_millis(10));
        let mut participant = timer.new_participant();
        participant.tick().await;
        let timers = (0 .. 16).map(|_| participant.clone()).collect::<Vec<_>>();
        for (i, mut participant) in timers.into_iter().enumerate() {
            join_set.spawn(async move {
                if i == 7 {
                    let sleep_first = tokio::select! {
                        _ = participant.tick() => false,
                        _ = time::sleep(Duration::from_micros(1)) => true,
                    };
                    assert!(sleep_first);
                    None
                } else {
                    Some(participant.tick().await)
                }
            });
        }
        let answer = participant.tick().await;
        let mut other_completion = 0;
        while let Some(maybe_alternative) = join_set.join_next().await {
            if let Some(alternative) = maybe_alternative.unwrap() {
                assert_eq!(answer, alternative);
                other_completion += 1;
            }
        }
        assert_eq!(other_completion, 15);
    }
}
