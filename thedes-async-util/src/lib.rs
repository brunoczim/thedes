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
        self.participants -= 1;
        if waiting {
            self.waiting -= 1;
            if self.participants <= self.waiting {
                debug_assert_eq!(self.participants, self.waiting);
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

    fn poll_interval(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        if self.participants == self.waiting {
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
}

#[derive(Debug)]
struct Shared {
    period: Duration,
    state: Mutex<State>,
}

#[derive(Debug)]
pub struct Timer {
    id: Id,
    last_known_tick: Instant,
    shared: Arc<Shared>,
}

impl Timer {
    pub fn new(period: Duration) -> Self {
        let start = Instant::now();
        let interval = time::interval(period);
        let mut state = State::new(interval, start);
        let id = state.new_participant();
        let shared = Shared { period, state: Mutex::new(state) };
        Self { id, last_known_tick: start, shared: Arc::new(shared) }
    }

    pub fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        let poll = self.with_state(|state| {
            state.poll_tick(cx, self.id, self.last_known_tick)
        });
        if let Poll::Ready(instant) = poll {
            self.last_known_tick = instant;
        }
        poll
    }

    pub fn tick(&mut self) -> Tick<'_> {
        Tick { timer: self }
    }

    pub fn period(&self) -> Duration {
        self.shared.period
    }

    pub fn last_tick(&self) -> Instant {
        self.with_state(|state| state.last_tick)
    }

    pub fn elapsed(&self) -> Duration {
        self.last_known_tick.elapsed()
    }

    pub fn time_left(&self) -> Duration {
        self.period().saturating_sub(self.elapsed())
    }

    fn with_state<F, T>(&self, scope: F) -> T
    where
        F: FnOnce(&mut State) -> T,
    {
        let mut state = self.shared.state.lock().expect("poisoned lock");
        scope(&mut state)
    }
}

impl Clone for Timer {
    fn clone(&self) -> Self {
        let id = self.with_state(|state| state.new_participant());
        Self {
            id,
            last_known_tick: self.last_known_tick,
            shared: self.shared.clone(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.with_state(|state| state.drop_participant(self.id))
    }
}

#[derive(Debug)]
pub struct Tick<'a> {
    timer: &'a mut Timer,
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

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::task::JoinSet;

    use crate::Timer;

    #[tokio::test]
    async fn sync_once() {
        let mut join_set = JoinSet::new();
        let mut timer = Timer::new(Duration::from_micros(100));
        let timers = (0 .. 16).map(|_| timer.clone()).collect::<Vec<_>>();
        for mut timer in timers {
            join_set.spawn(async move { timer.tick().await });
        }
        let answer = timer.tick().await;
        while let Some(alternative) = join_set.join_next().await {
            assert_eq!(answer, alternative.unwrap());
        }
    }

    #[tokio::test]
    async fn sync_twice() {
        let mut join_set = JoinSet::new();
        let mut timer = Timer::new(Duration::from_micros(100));
        let timers = (0 .. 16).map(|_| timer.clone()).collect::<Vec<_>>();
        for mut timer in timers {
            join_set.spawn(async move {
                let first = timer.tick().await;
                let second = timer.tick().await;
                (first, second)
            });
        }
        let first_answer = timer.tick().await;
        let second_answer = timer.tick().await;
        while let Some(alternative) = join_set.join_next().await {
            let (first_alternative, second_alternative) = alternative.unwrap();
            assert_eq!(first_answer, first_alternative);
            assert_eq!(second_answer, second_alternative);
        }
    }

    #[tokio::test]
    async fn sync_twice_with_novices() {
        let mut join_set = JoinSet::new();
        let mut timer = Timer::new(Duration::from_micros(100));
        let timers = (0 .. 16).map(|_| timer.clone()).collect::<Vec<_>>();
        for mut timer in timers {
            join_set.spawn(async move {
                let first = timer.tick().await;
                let second = timer.tick().await;
                (Some(first), second)
            });
        }
        let first_answer = timer.tick().await;
        let timers = (0 .. 7).map(|_| timer.clone()).collect::<Vec<_>>();
        for mut timer in timers {
            join_set.spawn(async move {
                let second = timer.tick().await;
                (None, second)
            });
        }
        let second_answer = timer.tick().await;
        while let Some(alternative) = join_set.join_next().await {
            let (maybe_first_alternative, second_alternative) =
                alternative.unwrap();
            if let Some(first_alternative) = maybe_first_alternative {
                assert_eq!(first_answer, first_alternative);
            }
            assert_eq!(second_answer, second_alternative);
        }
    }

    #[tokio::test]
    async fn sync_thrice_with_novices_and_leavos() {
        let mut join_set = JoinSet::new();
        let mut timer = Timer::new(Duration::from_micros(100));
        let timers = (0 .. 16).map(|_| timer.clone()).collect::<Vec<_>>();
        for (i, mut timer) in timers.into_iter().enumerate() {
            join_set.spawn(async move {
                let first = timer.tick().await;
                let second = timer.tick().await;
                let third = if i < 4 { None } else { Some(timer.tick().await) };
                (Some(first), second, third)
            });
        }
        let first_answer = timer.tick().await;
        let timers = (0 .. 7).map(|_| timer.clone()).collect::<Vec<_>>();
        for mut timer in timers {
            join_set.spawn(async move {
                let second = timer.tick().await;
                let third = timer.tick().await;
                (None, second, Some(third))
            });
        }
        let second_answer = timer.tick().await;
        let third_answer = timer.tick().await;
        while let Some(alternative) = join_set.join_next().await {
            let (
                maybe_first_alternative,
                second_alternative,
                maybe_third_alternative,
            ) = alternative.unwrap();
            if let Some(first_alternative) = maybe_first_alternative {
                assert_eq!(first_answer, first_alternative);
            }
            assert_eq!(second_answer, second_alternative);
            if let Some(third_alternative) = maybe_third_alternative {
                assert_eq!(third_answer, third_alternative);
            }
        }
    }

    #[tokio::test]
    async fn sync_thrice_with_novices_and_leavos_mixed() {
        let mut join_set = JoinSet::new();
        let mut timer = Timer::new(Duration::from_micros(100));
        let timers = (0 .. 16).map(|_| timer.clone()).collect::<Vec<_>>();
        for (i, mut timer) in timers.into_iter().enumerate() {
            join_set.spawn(async move {
                let first = timer.tick().await;
                let (second, third) = if i < 13 {
                    let second = timer.tick().await;
                    let third =
                        if i < 4 { None } else { Some(timer.tick().await) };
                    (Some(second), third)
                } else {
                    (None, None)
                };
                (Some(first), second, third)
            });
        }
        let first_answer = timer.tick().await;
        let timers = (0 .. 7).map(|_| timer.clone()).collect::<Vec<_>>();
        for mut timer in timers {
            join_set.spawn(async move {
                let second = timer.tick().await;
                let third = timer.tick().await;
                (None, Some(second), Some(third))
            });
        }
        let second_answer = timer.tick().await;
        let third_answer = timer.tick().await;
        while let Some(alternative) = join_set.join_next().await {
            let (
                maybe_first_alternative,
                maybe_second_alternative,
                maybe_third_alternative,
            ) = alternative.unwrap();
            if let Some(first_alternative) = maybe_first_alternative {
                assert_eq!(first_answer, first_alternative);
            }
            if let Some(second_alternative) = maybe_second_alternative {
                assert_eq!(second_answer, second_alternative);
            }
            if let Some(third_alternative) = maybe_third_alternative {
                assert_eq!(third_answer, third_alternative);
            }
        }
    }

    #[tokio::test]
    async fn sync_twice_only_one_left() {
        let mut join_set = JoinSet::new();
        let mut timer = Timer::new(Duration::from_micros(100));
        let timers = (0 .. 16).map(|_| timer.clone()).collect::<Vec<_>>();
        for mut timer in timers {
            join_set.spawn(async move { timer.tick().await });
        }
        let first_answer = timer.tick().await;
        let _second_answer = timer.tick().await;
        while let Some(alternative) = join_set.join_next().await {
            assert_eq!(first_answer, alternative.unwrap());
        }
    }
}
