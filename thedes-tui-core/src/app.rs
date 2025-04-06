use std::sync::{Arc, atomic::Ordering::*};

use thedes_async_util::{
    non_blocking::spsc::watch::{AtomicMessage, MessageBox},
    timer::TickSession,
};
use tokio_util::sync::CancellationToken;

use crate::{
    grapheme,
    input::EventReader,
    runtime::{self},
    screen::CanvasHandle,
};

#[derive(Debug)]
#[non_exhaustive]
pub struct App {
    pub tick_session: TickSession,
    pub canvas: CanvasHandle,
    pub events: EventReader,
    pub grapheme_registry: grapheme::Registry,
    pub cancel_token: CancellationToken,
}

impl App {
    pub(crate) fn run<F, A>(
        self,
        join_set: &mut runtime::JoinSet,
        scope: F,
    ) -> Arc<MessageBox<A::Output>>
    where
        F: FnOnce(Self) -> A,
        A: Future + Send + 'static,
        A::Output: Send + 'static,
    {
        let output = Arc::new(MessageBox::empty());
        let future = scope(self);
        join_set.spawn({
            let output = output.clone();
            async move {
                output.store(future.await, Relaxed);
                Ok(())
            }
        });
        output
    }
}
