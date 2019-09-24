use std::{
    thread,
    time::{Duration, Instant},
};

pub use self::TickExec::*;

pub enum TickExec<T> {
    Stop(T),
    Continue,
}

pub fn tick<F, T, E>(interval: Duration, mut exec: F) -> Result<T, E>
where
    F: FnMut() -> Result<TickExec<T>, E>,
{
    let then = Instant::now();
    let mut correction = Duration::new(0, 0);

    loop {
        if let Stop(ret) = exec()? {
            break Ok(ret);
        }

        let duration = interval - (then.elapsed() - correction);
        thread::sleep(duration);
        correction += duration;
    }
}
