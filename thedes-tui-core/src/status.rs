use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering::*},
};

use crate::geometry::CoordPair;

#[derive(Debug, Clone)]
pub struct Status {
    is_blocked: Arc<AtomicBool>,
}

impl Status {
    pub fn new() -> Self {
        Self { is_blocked: Arc::new(AtomicBool::new(false)) }
    }

    pub fn set_blocked(&self, is_blocked: bool) {
        self.is_blocked.store(is_blocked, Release);
    }

    pub fn sizes_would_block(
        &self,
        canvas_size: CoordPair,
        term_size: CoordPair,
    ) -> bool {
        canvas_size.zip2(term_size).any(|(canvas, term)| canvas + 2 > term)
    }

    pub fn set_blocked_from_sizes(
        &self,
        canvas_size: CoordPair,
        term_size: CoordPair,
    ) {
        self.set_blocked(self.sizes_would_block(canvas_size, term_size));
    }

    pub fn is_blocked(&self) -> bool {
        self.is_blocked.load(Acquire)
    }
}
