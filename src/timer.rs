use std::{
    thread,
    time::{Duration, Instant},
};

pub use self::NextExec::*;

/// A specification on what to do next execution.
pub enum NextExec<T> {
    /// Stop and return this value.
    Stop(T),
    /// Continue executing.
    Continue,
}

/// Execute the function `exec` every given `interval` approximately.
pub fn tick<F, T, E>(interval: Duration, mut exec: F) -> Result<T, E>
where
    F: FnMut() -> Result<NextExec<T>, E>,
{
    let then = Instant::now();
    let mut correction = Duration::new(0, 0);

    loop {
        if let Stop(ret) = exec()? {
            break Ok(ret);
        }

        if let Some(time) = interval.checked_sub(then.elapsed() - correction) {
            thread::sleep(time);
            correction += time;
        }
    }
}
