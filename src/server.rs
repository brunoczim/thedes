use crate::map::{Coord, Map};
use gardiz::coord::Vec2;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Request {
    LoadChunk(Vec2<Coord>),
}

#[derive(Debug)]
struct Server {
    map: Map,
    requests: mpsc::Receiver<Request>,
}

impl Server {
    fn event_loop(&mut self) {
        todo!()
    }
}
