use std::ops::Not;

pub trait Cancellation<I> {
    type Output;

    fn is_cancellable(&self) -> bool;

    fn is_cancelling(&self) -> bool;

    fn set_cancelling(&mut self, is_it: bool);

    fn make_output(&self, item: I) -> Self::Output;
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NonCancellable;

impl<I> Cancellation<I> for NonCancellable {
    type Output = I;

    fn is_cancellable(&self) -> bool {
        false
    }

    fn is_cancelling(&self) -> bool {
        false
    }

    fn set_cancelling(&mut self, _is_it: bool) {}

    fn make_output(&self, item: I) -> Self::Output {
        item
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Cancellable {
    is_cancelling: bool,
}

impl Cancellable {
    pub fn new(initial_cancelling: bool) -> Self {
        Self { is_cancelling: initial_cancelling }
    }
}

impl<I> Cancellation<I> for Cancellable {
    type Output = Option<I>;

    fn is_cancellable(&self) -> bool {
        true
    }

    fn is_cancelling(&self) -> bool {
        self.is_cancelling
    }

    fn set_cancelling(&mut self, is_it: bool) {
        self.is_cancelling = is_it;
    }

    fn make_output(&self, item: I) -> Self::Output {
        self.is_cancelling.not().then_some(item)
    }
}
