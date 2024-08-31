use std::{
    ops::{Add, Mul, Sub},
    rc::Rc,
    sync::Arc,
};

pub trait Signal<T> {
    type Sample;

    fn at(&self, time: T) -> Self::Sample;
}

impl<'a, S, T> Signal<T> for &'a S
where
    S: Signal<T> + ?Sized,
{
    type Sample = S::Sample;

    fn at(&self, time: T) -> Self::Sample {
        (**self).at(time)
    }
}

impl<'a, S, T> Signal<T> for &'a mut S
where
    S: Signal<T> + ?Sized,
{
    type Sample = S::Sample;

    fn at(&self, time: T) -> Self::Sample {
        (**self).at(time)
    }
}

impl<S, T> Signal<T> for Box<S>
where
    S: Signal<T> + ?Sized,
{
    type Sample = S::Sample;

    fn at(&self, time: T) -> Self::Sample {
        (**self).at(time)
    }
}

impl<S, T> Signal<T> for Rc<S>
where
    S: Signal<T> + ?Sized,
{
    type Sample = S::Sample;

    fn at(&self, time: T) -> Self::Sample {
        (**self).at(time)
    }
}

impl<S, T> Signal<T> for Arc<S>
where
    S: Signal<T> + ?Sized,
{
    type Sample = S::Sample;

    fn at(&self, time: T) -> Self::Sample {
        (**self).at(time)
    }
}

pub trait SignalExt<T>: Signal<T> {
    fn by_ref(&self) -> &Self {
        self
    }

    fn with_speed(self, speed: T) -> WithSpeed<Self, T>
    where
        Self: Sized,
    {
        WithSpeed { inner: self, speed }
    }

    fn with_volume(self, volume: T) -> WithVolume<Self, T>
    where
        Self: Sized,
    {
        WithVolume { inner: self, volume }
    }

    fn compose<R>(self, inner: R) -> Compose<Self, R>
    where
        Self: Sized,
    {
        Compose { outer: self, inner }
    }
}

impl<S, T> SignalExt<T> for S where S: Signal<T> + ?Sized {}

#[derive(Debug, Clone)]
pub struct SignalFn<F>(pub F);

impl<F, T, U> Signal<T> for SignalFn<F>
where
    F: Fn(T) -> U,
{
    type Sample = U;

    fn at(&self, time: T) -> Self::Sample {
        (self.0)(time)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sin;

impl Signal<f32> for Sin {
    type Sample = f32;

    fn at(&self, time: f32) -> Self::Sample {
        time.sin()
    }
}

impl Signal<f64> for Sin {
    type Sample = f64;

    fn at(&self, time: f64) -> Self::Sample {
        time.sin()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cos;

impl Signal<f32> for Cos {
    type Sample = f32;

    fn at(&self, time: f32) -> Self::Sample {
        time.cos()
    }
}

impl Signal<f64> for Cos {
    type Sample = f64;

    fn at(&self, time: f64) -> Self::Sample {
        time.cos()
    }
}

#[derive(Debug, Clone)]
pub struct WithSpeed<S, T> {
    inner: S,
    speed: T,
}

impl<S, T> Signal<T> for WithSpeed<S, T>
where
    S: Signal<T>,
    T: for<'a> Mul<&'a T, Output = T>,
{
    type Sample = S::Sample;

    fn at(&self, time: T) -> Self::Sample {
        self.inner.at(time * &self.speed)
    }
}

#[derive(Debug, Clone)]
pub struct WithVolume<S, T> {
    inner: S,
    volume: T,
}

impl<S, T> Signal<T> for WithVolume<S, T>
where
    S: Signal<T>,
    T: for<'a> Add<&'a T, Output = T>,
{
    type Sample = S::Sample;

    fn at(&self, time: T) -> Self::Sample {
        self.inner.at(time + &self.volume)
    }
}

#[derive(Debug, Clone)]
pub struct Compose<S, R> {
    outer: S,
    inner: R,
}

impl<S, R, T> Signal<T> for Compose<S, R>
where
    R: Signal<T>,
    S: Signal<R::Sample>,
{
    type Sample = S::Sample;

    fn at(&self, time: T) -> Self::Sample {
        self.outer.at(self.inner.at(time))
    }
}

#[derive(Debug, Clone)]
pub struct Switch<S, R, T> {
    pre: S,
    post: R,
    condition: T,
}

impl<S, R, T> Signal<T> for Switch<S, R, T>
where
    S: Signal<T>,
    R: Signal<T, Sample = S::Sample>,
    T: PartialOrd + for<'a> Sub<&'a T, Output = T>,
{
    type Sample = S::Sample;

    fn at(&self, time: T) -> Self::Sample {
        if time >= self.condition {
            self.post.at(time - &self.condition)
        } else {
            self.pre.at(time)
        }
    }
}
