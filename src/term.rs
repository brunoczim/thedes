use crate::{
    backend::{Backend, DefaultBackend},
    error::GameResult,
    orient::Coord2D,
    render::MIN_SCREEN,
};
use std::{
    thread,
    time::{Duration, Instant},
};

pub use self::Tick::*;

/// A specification on what to do next execution.
pub enum Tick<T> {
    /// Stop and return this value.
    Stop(T),
    /// Continue executing.
    Continue,
}

#[derive(Debug)]
pub struct Term<B = DefaultBackend>
where
    B: Backend,
{
    screen_size: Coord2D,
    interval: Duration,
    then: Instant,
    correction: Duration,
    backend: B,
}

impl<B> Term<B>
where
    B: Backend,
{
    pub fn evt_loop<F, T>(
        mut backend: B,
        interval: Duration,
        start: F,
    ) -> GameResult<T>
    where
        F: FnMut(&mut Self) -> GameResult<Tick<T>>,
    {
        let mut this = Self {
            screen_size: backend.screen_size()?,
            interval,
            then: Instant::now(),
            correction: Duration::new(0, 0),
            backend,
        };

        this.call(start)
    }

    pub fn screen_size(&mut self) -> Coord2D {
        self.screen_size
    }

    pub fn call<F, T>(&mut self, mut fun: F) -> GameResult<T>
    where
        F: FnMut(&mut Self) -> GameResult<Tick<T>>,
    {
        loop {
            if let Stop(ret) = self.tick(&mut fun)? {
                break Ok(ret);
            }

            let diff = self.then.elapsed() - self.correction;
            if let Some(time) = self.interval.checked_sub(diff) {
                thread::sleep(time);
                self.correction += time;
            }
        }
    }

    fn tick<F, T>(&mut self, mut fun: F) -> GameResult<Tick<T>>
    where
        F: FnMut(&mut Self) -> GameResult<Tick<T>>,
    {
        self.check_screen_size()?;
        fun(self)
    }

    fn check_screen_size(&mut self) -> GameResult<bool> {
        let mut new_screen = self.backend.screen_size()?;

        if new_screen.x < MIN_SCREEN.x || new_screen.y < MIN_SCREEN.y {
            self.backend.clear_screen()?;
            self.backend.goto(Coord2D { x: 0, y: 0 })?;
            write!(
                self.backend,
                "RESIZE {:?},{:?}",
                MIN_SCREEN.x, MIN_SCREEN.y
            )?;

            while new_screen.x < MIN_SCREEN.x || new_screen.y < MIN_SCREEN.y {
                new_screen = self.backend.screen_size()?
            }
        }

        if new_screen != self.screen_size {
            self.screen_size = new_screen;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
