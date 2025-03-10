use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use pin_project::pin_project;

pub fn sleep(duration: Duration) -> Sleep {
    Sleep::wrap(tokio::time::sleep(duration))
}

pub fn interval(period: Duration) -> Interval {
    Interval::wrap(tokio::time::interval(period))
}

#[derive(Debug)]
#[pin_project]
pub struct Sleep {
    #[pin]
    inner: tokio::time::Sleep,
}

impl Sleep {
    fn wrap(inner: tokio::time::Sleep) -> Self {
        Self { inner }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

#[derive(Debug)]
pub struct Interval {
    inner: tokio::time::Interval,
}

impl Interval {
    fn wrap(inner: tokio::time::Interval) -> Self {
        Self { inner }
    }

    pub async fn tick(&mut self) {
        self.inner.tick().await;
    }
}
