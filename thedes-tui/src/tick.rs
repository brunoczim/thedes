use crate::app::App;

#[derive(Debug)]
pub struct TickEvent<'a> {
    stop_requested: bool,
    app: &'a mut App,
}

impl<'a> TickEvent<'a> {
    pub(crate) fn new(app: &'a mut App) -> Self {
        Self { stop_requested: false, app }
    }

    pub(crate) fn stop_requested(&self) -> bool {
        self.stop_requested
    }

    pub fn request_stop(&mut self) {
        self.stop_requested = true;
    }
}
