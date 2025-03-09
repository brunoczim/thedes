use crate::wasm::extensions::callback;
use std::{future::Future, pin::Pin, task};

use futures::stream::Stream;

macro_rules! sync_multi {
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

macro_rules! async_multi {
    ($self:expr, $callback:expr) => {{
        let (notifier, inner_listener) = callback::shared::channel();

        let handler = Box::new(move |event_data| {
            let future = $callback(event_data);
            let notifier = notifier.clone();
            let handler_future = Box::pin(async move {
                let data = future.await;
                notifier.send(data);
            });
            handler_future as AsyncCbHandlerFuture
        });
        let ret = ($self.register_fn)(handler as AsyncCbHandler<_>);

        (ret, Listener::new(inner_listener))
    }};
}

pub type SyncCbHandler<'cb, T> = Box<dyn FnMut(T) + 'cb>;

pub type AsyncCbHandlerFuture<'fut> = Pin<Box<dyn Future<Output = ()> + 'fut>>;

pub type AsyncCbHandler<'cb, 'fut, T> =
    Box<dyn FnMut(T) -> AsyncCbHandlerFuture<'fut> + 'cb>;

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
        C: FnMut(T) -> V + 'cb,
        V: 'cb,
    {
        let (_, listener) = self.listen_returning(callback);
        listener
    }

    pub fn listen_mut<'cb, C, T, V>(&mut self, callback: C) -> Listener<V>
    where
        F: FnMut(SyncCbHandler<'cb, T>),
        C: FnMut(T) -> V + 'cb,
        V: 'cb,
    {
        let (_, listener) = self.listen_mut_returning(callback);
        listener
    }

    pub fn listen_ref<'cb, C, T, V>(&mut self, callback: C) -> Listener<V>
    where
        F: Fn(SyncCbHandler<'cb, T>),
        C: FnMut(T) -> V + 'cb,
        V: 'cb,
    {
        let (_, listener) = self.listen_ref_returning(callback);
        listener
    }

    pub fn listen_returning<'cb, C, T, U, V>(
        self,
        mut callback: C,
    ) -> (U, Listener<V>)
    where
        F: FnOnce(SyncCbHandler<'cb, T>) -> U,
        C: FnMut(T) -> V + 'cb,
        V: 'cb,
    {
        sync_multi!(self, callback)
    }

    pub fn listen_mut_returning<'cb, C, T, U, V>(
        &mut self,
        mut callback: C,
    ) -> (U, Listener<V>)
    where
        F: FnMut(SyncCbHandler<'cb, T>) -> U,
        C: FnMut(T) -> V + 'cb,
        V: 'cb,
    {
        sync_multi!(self, callback)
    }

    pub fn listen_ref_returning<'cb, C, T, U, V>(
        &self,
        mut callback: C,
    ) -> (U, Listener<V>)
    where
        F: Fn(SyncCbHandler<'cb, T>) -> U,
        C: FnMut(T) -> V + 'cb,
        V: 'cb,
    {
        sync_multi!(self, callback)
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
        C: FnMut(T) -> A + 'cb,
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
        C: FnMut(T) -> A + 'cb,
        A: Future + 'fut,
    {
        let (_, listener) = self.listen_mut_returning(callback);
        listener
    }

    pub fn listen_ref<'cb, 'fut, C, T, A>(
        &self,
        callback: C,
    ) -> Listener<A::Output>
    where
        'fut: 'cb,
        F: Fn(AsyncCbHandler<'cb, 'fut, T>),
        C: FnMut(T) -> A + 'cb,
        A: Future + 'fut,
    {
        let (_, listener) = self.listen_ref_returning(callback);
        listener
    }

    pub fn listen_returning<'cb, 'fut, C, T, U, A>(
        self,
        mut callback: C,
    ) -> (U, Listener<A::Output>)
    where
        'fut: 'cb,
        F: FnOnce(AsyncCbHandler<'cb, 'fut, T>) -> U,
        C: FnMut(T) -> A + 'cb,
        A: Future + 'fut,
    {
        async_multi!(self, callback)
    }

    pub fn listen_mut_returning<'cb, 'fut, C, T, U, A>(
        &mut self,
        mut callback: C,
    ) -> (U, Listener<A::Output>)
    where
        'fut: 'cb,
        F: FnMut(AsyncCbHandler<'cb, 'fut, T>) -> U,
        C: FnMut(T) -> A + 'cb,
        A: Future + 'fut,
    {
        async_multi!(self, callback)
    }

    pub fn listen_ref_returning<'cb, 'fut, C, T, U, A>(
        &self,
        mut callback: C,
    ) -> (U, Listener<A::Output>)
    where
        'fut: 'cb,
        F: Fn(AsyncCbHandler<'cb, 'fut, T>) -> U,
        C: FnMut(T) -> A + 'cb,
        A: Future + 'fut,
    {
        async_multi!(self, callback)
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

    pub fn listen_next<'this>(&'this self) -> ListenNext<'this, T> {
        ListenNext::new(self)
    }

    fn generic_poll(
        &self,
        ctx: &mut task::Context<'_>,
    ) -> task::Poll<Result<T, callback::Cancelled>> {
        match self.inner.receive() {
            Some(output) => task::Poll::Ready(output),
            None => {
                self.inner.subscribe(ctx.waker());
                task::Poll::Pending
            },
        }
    }
}

impl<T> Stream for Listener<T> {
    type Item = T;

    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        self.generic_poll(ctx).map(Result::ok)
    }
}

#[derive(Debug)]
pub struct ListenNext<'list, T> {
    listener: &'list Listener<T>,
}

impl<'list, T> ListenNext<'list, T> {
    fn new(listener: &'list Listener<T>) -> Self {
        Self { listener }
    }
}

impl<'list, T> Future for ListenNext<'list, T> {
    type Output = Result<T, callback::Cancelled>;

    fn poll(
        self: Pin<&mut Self>,
        ctx: &mut task::Context<'_>,
    ) -> task::Poll<Self::Output> {
        self.listener.generic_poll(ctx)
    }
}
