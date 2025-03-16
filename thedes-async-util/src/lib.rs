use tokio::time::{
    Duration,
    Instant,
    Interval,
    MissedTickBehavior,
    interval_at,
};

#[derive(Debug)]
pub struct Timer {
    start: Instant,
    interval: Interval,
}

impl Timer {
    pub fn new_at(start: Instant, period: Duration) -> Self {
        let mut interval = interval_at(start, period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        Self { start, interval }
    }

    pub fn new_immediate(period: Duration) -> Self {
        Self::new_at(Instant::now(), period)
    }

    pub async fn tick(&mut self) {
        self.interval.tick().await;
    }

    pub fn start(&self) -> Instant {
        self.start
    }

    pub fn period(&self) -> Duration {
        self.interval.period()
    }

    pub fn elapsed(&self) -> Duration {
        let since_start = self.start.elapsed();
        let since_start_nanos = since_start.as_nanos();
        let period_nanos = self.period().as_nanos();
        let since_tick_nanos = since_start_nanos % period_nanos;
        Duration::new(
            (since_tick_nanos / 1_000_000_000) as u64,
            (since_tick_nanos % 1_000_000_000) as u32,
        )
    }

    pub fn time_left(&self) -> Duration {
        let since_start = self.start.elapsed();
        let since_start_nanos = since_start.as_nanos();
        let period_nanos = self.period().as_nanos();
        let left_nanos = period_nanos - since_start_nanos % period_nanos;
        Duration::new(
            (left_nanos / 1_000_000_000) as u64,
            (left_nanos % 1_000_000_000) as u32,
        )
    }
}

impl Clone for Timer {
    fn clone(&self) -> Self {
        Self::new_at(self.start, self.interval.period())
    }
}
