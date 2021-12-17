pub mod block;
pub mod ground;
pub mod biome;
pub mod language;
pub mod thede;
pub mod human;
pub mod map;

use map::Map;

#[derive(Debug)]
struct Server {
    map: Map,
}

impl Server {
    fn event_loop(&mut self) {
        todo!()
    }
}
