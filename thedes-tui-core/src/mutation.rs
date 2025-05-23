use std::fmt;

pub trait Mutable: Send + Sync {
    type Error: std::error::Error + Send + Sync;
}

pub trait Mutation<T>: fmt::Debug + Send + Sync + 'static
where
    T: Mutable,
{
    fn mutate(self, target: T) -> Result<T, T::Error>;
}

impl<M, T> Mutation<T> for Box<M>
where
    M: BoxedMutation<T> + ?Sized,
    T: Mutable,
{
    fn mutate(self, target: T) -> Result<T, <T as Mutable>::Error> {
        self.mutate_boxed(target)
    }
}

pub trait BoxedMutation<T>: Mutation<T>
where
    T: Mutable,
{
    fn mutate_boxed(self: Box<Self>, target: T) -> Result<T, T::Error>;
}

impl<M, T> BoxedMutation<T> for M
where
    M: Mutation<T>,
    T: Mutable,
{
    fn mutate_boxed(
        self: Box<Self>,
        target: T,
    ) -> Result<T, <T as Mutable>::Error> {
        (*self).mutate(target)
    }
}

pub trait MutationExt<T>: Mutation<T>
where
    T: Mutable,
{
    fn then<N>(self, after: N) -> Then<Self, N>
    where
        Self: Sized,
        N: Mutation<T>,
    {
        Then { before: self, after }
    }
}

impl<M, T> MutationExt<T> for M
where
    T: Mutable,
    M: Mutation<T>,
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Then<M, N> {
    before: M,
    after: N,
}

impl<M, N, T> Mutation<T> for Then<M, N>
where
    T: Mutable,
    M: Mutation<T>,
    N: Mutation<T>,
{
    fn mutate(self, target: T) -> Result<T, T::Error> {
        self.after.mutate(self.before.mutate(target)?)
    }
}

#[derive(Clone, Copy)]
pub struct MutationFn<F>(pub F);

impl<F> fmt::Debug for MutationFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MutationFn").field(&(&self.0 as *const F)).finish()
    }
}

impl<F, T> Mutation<T> for MutationFn<F>
where
    T: Mutable,
    F: FnOnce(T) -> Result<T, T::Error> + Send + Sync + 'static,
{
    fn mutate(self, target: T) -> Result<T, T::Error> {
        (self.0)(target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Id;

impl<T> Mutation<T> for Id
where
    T: Mutable,
{
    fn mutate(self, target: T) -> Result<T, T::Error> {
        Ok(target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Set<T>(pub T);

impl<T> Mutation<T> for Set<T>
where
    T: Mutable + fmt::Debug + 'static,
{
    fn mutate(self, _target: T) -> Result<T, T::Error> {
        let Set(output) = self;
        Ok(output)
    }
}
