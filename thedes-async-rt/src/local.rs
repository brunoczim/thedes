use std::{
    cell::RefCell,
    mem,
    pin::Pin,
    task::{Context, Poll},
    thread::LocalKey,
};

use pin_project::pin_project;

#[derive(Debug)]
pub struct DynScoped<T> {
    target: RefCell<T>,
}

impl<T> DynScoped<T> {
    pub const fn new(default: T) -> Self {
        Self { target: RefCell::new(default) }
    }

    fn swap(&self, new_value: &mut T) {
        mem::swap(&mut *self.target.borrow_mut(), new_value);
    }

    fn with<F, U>(&self, scope: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        scope(&*self.target.borrow())
    }
}

pub trait LocalScopeExt<T> {
    fn with_scoped<F, U>(&'static self, scope: F) -> U
    where
        F: FnOnce(&T) -> U;

    fn cloned(&'static self) -> T
    where
        T: Clone;

    fn enter(&'static self, new_value: T) -> ScopeGuard<T>;

    fn enter_async<A>(
        &'static self,
        new_value: T,
        future: A,
    ) -> AsyncScopeGuard<T, A>
    where
        A: Future;
}

impl<T> LocalScopeExt<T> for LocalKey<DynScoped<T>> {
    fn with_scoped<F, U>(&'static self, scope: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        self.with(|scoped| scoped.with(scope))
    }

    fn cloned(&'static self) -> T
    where
        T: Clone,
    {
        self.with_scoped(T::clone)
    }

    fn enter(&'static self, mut new_value: T) -> ScopeGuard<T> {
        self.with(|scoped| scoped.swap(&mut new_value));
        ScopeGuard::new(self, new_value)
    }

    fn enter_async<A>(
        &'static self,
        new_value: T,
        future: A,
    ) -> AsyncScopeGuard<T, A>
    where
        A: Future,
    {
        AsyncScopeGuard::new(self, new_value, future)
    }
}

#[derive(Debug)]
pub struct ScopeGuard<T: 'static> {
    resource: &'static LocalKey<DynScoped<T>>,
    old_value: T,
}

impl<T: 'static> ScopeGuard<T> {
    fn new(resource: &'static LocalKey<DynScoped<T>>, old_value: T) -> Self {
        Self { resource, old_value }
    }
}

impl<T: 'static> Drop for ScopeGuard<T> {
    fn drop(&mut self) {
        self.resource.with(|scoped| scoped.swap(&mut self.old_value));
    }
}

#[derive(Debug)]
struct PollScopeGuard<'a, T: 'static> {
    resource: &'static LocalKey<DynScoped<T>>,
    value: &'a mut T,
}

impl<'a, T: 'static> PollScopeGuard<'a, T> {
    fn new(
        resource: &'static LocalKey<DynScoped<T>>,
        value: &'a mut T,
    ) -> Self {
        Self { resource, value }
    }
}

impl<'a, T: 'static> Drop for PollScopeGuard<'a, T> {
    fn drop(&mut self) {
        self.resource.with(|scoped| scoped.swap(self.value));
    }
}

#[derive(Debug)]
#[pin_project]
pub struct AsyncScopeGuard<T: 'static, A> {
    resource: &'static LocalKey<DynScoped<T>>,
    new_value: T,
    #[pin]
    future: A,
}

impl<T: 'static, A> AsyncScopeGuard<T, A> {
    fn new(
        resource: &'static LocalKey<DynScoped<T>>,
        new_value: T,
        future: A,
    ) -> Self {
        Self { resource, new_value, future }
    }
}

impl<T: 'static, A> Future for AsyncScopeGuard<T, A>
where
    A: Future,
{
    type Output = A::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        this.resource.with(|scoped| scoped.swap(&mut this.new_value));
        let _guard = PollScopeGuard::new(this.resource, &mut this.new_value);
        this.future.poll(cx)
    }
}
