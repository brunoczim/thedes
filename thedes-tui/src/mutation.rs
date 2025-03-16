pub trait Mutable {
    type Error: std::error::Error + Send + Sync;
}

pub trait Mutation<T>
where
    T: Mutable,
{
    fn mutate(self, target: T) -> Result<T, T::Error>;
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

impl<F, T> Mutation<T> for F
where
    T: Mutable,
    F: FnOnce(T) -> Result<T, T::Error>,
{
    fn mutate(self, target: T) -> Result<T, T::Error> {
        self(target)
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
    T: Mutable,
{
    fn mutate(self, _target: T) -> Result<T, T::Error> {
        let Set(output) = self;
        Ok(output)
    }
}
