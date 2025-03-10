use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::Stream;
use pin_project::{pin_project, pinned_drop};
use wasm_bindgen::{
    JsValue,
    prelude::{Closure, wasm_bindgen},
};

use super::extensions::callback;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "setTimeout")]
    fn set_timeout(function: &JsValue, milliseconds: i32) -> JsValue;
    #[wasm_bindgen(js_name = "clearTimeout")]
    fn clear_timeout(timeout_id: &JsValue);

    #[wasm_bindgen(js_name = "setInterval")]
    fn set_interval(function: &JsValue, milliseconds: i32) -> JsValue;
    #[wasm_bindgen(js_name = "clearInterval")]
    fn clear_interval(interval_id: &JsValue);
}

pub fn sleep(duration: Duration) -> Sleep {
    let rounding = (duration.as_nanos() + 499_999) / 1_000_000;
    let milliseconds =
        i32::try_from(duration.as_millis().saturating_add(rounding))
            .unwrap_or(i32::MAX);

    let register = callback::once::SyncRegister::new(|callback| {
        let closure = Closure::once_into_js(move || callback(()));
        let timeout_id = set_timeout(&closure, milliseconds);
        (timeout_id, closure)
    });

    let ((id, closure), listener) = register.listen_returning(|()| ());

    Sleep::new(listener, id, closure)
}

pub fn interval(period: Duration) -> Interval {
    let rounding = (period.as_nanos() + 499_999) / 1_000_000;
    let milliseconds =
        i32::try_from(period.as_millis().saturating_add(rounding))
            .unwrap_or(i32::MAX);

    let register = callback::multi::SyncRegister::new(|mut callback| {
        let boxed_callback = Box::new(move || callback(()));
        let closure =
            Closure::wrap(boxed_callback as Box<dyn FnMut()>).into_js_value();
        let timeout_id = set_interval(&closure, milliseconds);
        (timeout_id, closure)
    });

    let ((id, closure), listener) = register.listen_returning(|()| ());

    Interval::new(listener, id, closure)
}

#[derive(Debug)]
#[pin_project(PinnedDrop)]
pub struct Sleep {
    #[pin]
    listener: callback::once::Listener<()>,
    timeout_id: JsValue,
    _closure: JsValue,
}

impl Sleep {
    fn new(
        listener: callback::once::Listener<()>,
        timeout_id: JsValue,
        closure: JsValue,
    ) -> Self {
        Self { listener, timeout_id, _closure: closure }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().listener.poll(cx).map(|_| ())
    }
}

#[pinned_drop]
impl PinnedDrop for Sleep {
    fn drop(self: Pin<&mut Self>) {
        clear_timeout(&self.timeout_id);
    }
}

#[derive(Debug)]
pub struct Interval {
    listener: callback::multi::Listener<()>,
    interval_id: JsValue,
    _closure: JsValue,
}

impl Interval {
    fn new(
        listener: callback::multi::Listener<()>,
        interval_id: JsValue,
        closure: JsValue,
    ) -> Self {
        Self { listener, interval_id, _closure: closure }
    }

    pub async fn tick(&mut self) {
        let _ = self.listener.listen_next().await;
    }
}

impl Stream for Interval {
    type Item = ();

    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.listener).poll_next(ctx)
    }
}

impl Drop for Interval {
    fn drop(&mut self) {
        clear_interval(&self.interval_id);
    }
}
