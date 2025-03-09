use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::wasm::extensions::callback;

macro_rules! sync_once {
    ($self:expr, $callback:expr) => {{
        let (notifier, inner_listener) = callback::shared::channel();

        let handler = Box::new(move |event_data| {
            let data = $callback(event_data);
            notifier.send(data);
        });
        let ret = ($self.register_fn)(handler as SyncCbHandler<_>);

        (ret, Listener::new(inner_listener))
    }};
}

macro_rules! async_once {
    ($self:expr, $callback:expr) => {{
        let (notifier, inner_listener) = callback::shared::channel();

        let handler = Box::new(move |event_data| {
            let callback_future = $callback(event_data);
            let handler_future = Box::pin(async move {
                let data = callback_future.await;
                notifier.send(data);
            });
            handler_future as AsyncCbHandlerFuture
        });
        let ret = ($self.register_fn)(handler as AsyncCbHandler<_>);

        (ret, Listener::new(inner_listener))
    }};
}

pub type SyncCbHandler<'cb, T> = Box<dyn FnOnce(T) + 'cb>;

pub type AsyncCbHandlerFuture<'fut> = Pin<Box<dyn Future<Output = ()> + 'fut>>;

pub type AsyncCbHandler<'cb, 'fut, T> =
    Box<dyn FnOnce(T) -> AsyncCbHandlerFuture<'fut> + 'cb>;

#[derive(Debug, Clone, Copy)]
pub struct SyncRegister<F> {
    register_fn: F,
}

impl<F> SyncRegister<F> {
    pub fn new<'cb, T, U>(register_fn: F) -> Self
    where
        F: FnOnce(SyncCbHandler<'cb, T>) -> U,
    {
        Self { register_fn }
    }

    pub fn new_mut<'cb, T, U>(register_fn: F) -> Self
    where
        F: FnMut(SyncCbHandler<'cb, T>) -> U,
    {
        Self { register_fn }
    }

    pub fn new_ref<'cb, T, U>(register_fn: F) -> Self
    where
        F: Fn(SyncCbHandler<'cb, T>) -> U,
    {
        Self { register_fn }
    }

    pub fn listen<'cb, C, T, V>(self, callback: C) -> Listener<V>
    where
        F: FnOnce(SyncCbHandler<'cb, T>),
        C: FnOnce(T) -> V + 'cb,
        V: 'cb,
    {
        let (_, listener) = self.listen_returning(callback);
        listener
    }

    pub fn listen_mut<'cb, C, T, U, V>(&mut self, callback: C) -> Listener<V>
    where
        F: FnMut(SyncCbHandler<'cb, T>),
        C: FnOnce(T) -> V + 'cb,
        V: 'cb,
    {
        let (_, listener) = self.listen_mut_returning(callback);
        listener
    }

    pub fn listen_ref<'cb, C, T, U, V>(&self, callback: C) -> Listener<V>
    where
        F: Fn(SyncCbHandler<'cb, T>),
        C: FnOnce(T) -> V + 'cb,
        V: 'cb,
    {
        let (_, listener) = self.listen_ref_returning(callback);
        listener
    }

    pub fn listen_returning<'cb, C, T, U, V>(
        self,
        callback: C,
    ) -> (U, Listener<V>)
    where
        F: FnOnce(SyncCbHandler<'cb, T>) -> U,
        C: FnOnce(T) -> V + 'cb,
        V: 'cb,
    {
        sync_once!(self, callback)
    }

    pub fn listen_mut_returning<'cb, C, T, U, V>(
        &mut self,
        callback: C,
    ) -> (U, Listener<V>)
    where
        F: FnMut(SyncCbHandler<'cb, T>) -> U,
        C: FnOnce(T) -> V + 'cb,
        V: 'cb,
    {
        sync_once!(self, callback)
    }

    pub fn listen_ref_returning<'cb, C, T, U, V>(
        &self,
        callback: C,
    ) -> (U, Listener<V>)
    where
        F: Fn(SyncCbHandler<'cb, T>) -> U,
        C: FnOnce(T) -> V + 'cb,
        V: 'cb,
    {
        sync_once!(self, callback)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AsyncRegister<F> {
    register_fn: F,
}

impl<F> AsyncRegister<F> {
    pub fn new<'cb, 'fut, T, U>(register_fn: F) -> Self
    where
        'fut: 'cb,
        F: FnOnce(AsyncCbHandler<'cb, 'fut, T>) -> U,
    {
        Self { register_fn }
    }

    pub fn new_mut<'cb, 'fut, T, U>(register_fn: F) -> Self
    where
        'fut: 'cb,
        F: FnMut(AsyncCbHandler<'cb, 'fut, T>) -> U,
    {
        Self { register_fn }
    }

    pub fn new_ref<'cb, 'fut, T, U>(register_fn: F) -> Self
    where
        'fut: 'cb,
        F: Fn(AsyncCbHandler<'cb, 'fut, T>) -> U,
    {
        Self { register_fn }
    }

    pub fn listen<'cb, 'fut, C, T, A>(self, callback: C) -> Listener<A::Output>
    where
        'fut: 'cb,
        F: FnOnce(AsyncCbHandler<'cb, 'fut, T>),
        C: FnOnce(T) -> A + 'cb,
        A: Future + 'fut,
    {
        let (_, listener) = self.listen_returning(callback);
        listener
    }

    pub fn listen_mut<'cb, 'fut, C, T, A>(
        &mut self,
        callback: C,
    ) -> Listener<A::Output>
    where
        'fut: 'cb,
        F: FnMut(AsyncCbHandler<'cb, 'fut, T>),
        C: FnOnce(T) -> A + 'cb,
        A: Future + 'fut,
    {
        let (_, listener) = self.listen_mut_returning(callback);
        listener
    }

    pub fn listen_ref<'cb, 'fut, C, T, A>(
        &mut self,
        callback: C,
    ) -> Listener<A::Output>
    where
        'fut: 'cb,
        F: Fn(AsyncCbHandler<'cb, 'fut, T>),
        C: FnOnce(T) -> A + 'cb,
        A: Future + 'fut,
    {
        let (_, listener) = self.listen_ref_returning(callback);
        listener
    }

    pub fn listen_returning<'cb, 'fut, C, T, A, U>(
        self,
        callback: C,
    ) -> (U, Listener<A::Output>)
    where
        'fut: 'cb,
        F: FnOnce(AsyncCbHandler<'cb, 'fut, T>) -> U,
        C: FnOnce(T) -> A + 'cb,
        A: Future + 'fut,
    {
        async_once!(self, callback)
    }

    pub fn listen_mut_returning<'cb, 'fut, C, T, A, U>(
        &mut self,
        callback: C,
    ) -> (U, Listener<A::Output>)
    where
        'fut: 'cb,
        F: FnMut(AsyncCbHandler<'cb, 'fut, T>) -> U,
        C: FnOnce(T) -> A + 'cb,
        A: Future + 'fut,
    {
        async_once!(self, callback)
    }

    pub fn listen_ref_returning<'cb, 'fut, C, T, A, U>(
        &self,
        callback: C,
    ) -> (U, Listener<A::Output>)
    where
        'fut: 'cb,
        F: Fn(AsyncCbHandler<'cb, 'fut, T>) -> U,
        C: FnOnce(T) -> A + 'cb,
        A: Future + 'fut,
    {
        async_once!(self, callback)
    }
}

#[derive(Debug)]
pub struct Listener<T> {
    inner: callback::shared::Listener<T>,
}

impl<T> Listener<T> {
    fn new(inner: callback::shared::Listener<T>) -> Self {
        Self { inner }
    }
}

impl<T> Future for Listener<T> {
    type Output = Result<T, callback::Cancelled>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.receive() {
            Some(output) => Poll::Ready(output),
            None => {
                self.inner.subscribe(ctx.waker());
                Poll::Pending
            },
        }
    }
}
