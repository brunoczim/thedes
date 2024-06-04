use std::borrow::Cow;

pub mod menu;
pub mod info;
pub mod input;

pub trait Cancellability {
    fn cancel_state(&self) -> Option<bool>;

    fn set_cancel_state(&mut self, state: bool);

    fn cancel_label(&self) -> Cow<str>;
}

pub trait SelectionCancellability<O>: Cancellability {
    type Output;

    fn select(&self, item: O) -> Self::Output;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NonCancellable;

impl Cancellability for NonCancellable {
    fn cancel_state(&self) -> Option<bool> {
        None
    }

    fn set_cancel_state(&mut self, _state: bool) {}

    fn cancel_label(&self) -> Cow<str> {
        "".into()
    }
}

impl<O> SelectionCancellability<O> for NonCancellable {
    type Output = O;

    fn select(&self, item: O) -> Self::Output {
        item
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cancellable {
    label: String,
    selected: bool,
}

impl Default for Cancellable {
    fn default() -> Self {
        Self::new()
    }
}

impl Cancellable {
    pub fn new() -> Self {
        Self { label: "CANCEL".into(), selected: false }
    }

    pub fn selected(self) -> Self {
        Self { selected: true, ..self }
    }

    pub fn with_cancel_label(self, label: impl Into<String>) -> Self {
        Self { label: label.into(), ..self }
    }
}

impl Cancellability for Cancellable {
    fn cancel_state(&self) -> Option<bool> {
        Some(self.selected)
    }

    fn set_cancel_state(&mut self, state: bool) {
        self.selected = state;
    }

    fn cancel_label(&self) -> Cow<str> {
        Cow::Borrowed(&self.label)
    }
}

impl<O> SelectionCancellability<O> for Cancellable {
    type Output = Option<O>;

    fn select(&self, item: O) -> Self::Output {
        Some(item).filter(|_| !self.selected)
    }
}
