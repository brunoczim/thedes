use thedes_async_util::progress;
use thedes_domain::{geometry::CoordPair, map::Map};
use thiserror::Error;
use tokio::task;

use crate::random::PickedReproducibleRng;

use super::{Layer, LayerDistribution};

#[derive(Debug, Error)]
pub enum Error<L, Ld>
where
    L: std::error::Error,
    Ld: std::error::Error,
{
    #[error("Failed to manipulate layer")]
    Layer(#[source] L),
    #[error("Failed to manipulate layer distribution")]
    LayerDistribution(#[source] Ld),
}

#[derive(Debug)]
pub struct Generator {
    _priv: (),
}

impl Generator {
    pub fn new() -> Self {
        Self { _priv: () }
    }

    pub fn progress_goal(&self, map: &Map) -> usize {
        map.rect().map(usize::from).total_area()
    }

    pub async fn execute<L, Ld>(
        self,
        layer: &L,
        layer_distr: &Ld,
        map: &mut Map,
        rng: &mut PickedReproducibleRng,
        progress_logger: progress::Logger,
    ) -> Result<(), Error<L::Error, Ld::Error>>
    where
        L: Layer,
        L::Error: std::error::Error,
        Ld: LayerDistribution<Data = L::Data>,
        Ld::Error: std::error::Error,
    {
        progress_logger.set_status("generating point block");

        let map_rect = map.rect();
        for y in map_rect.top_left.y .. map_rect.bottom_right().y {
            for x in map_rect.top_left.x .. map_rect.bottom_right().x {
                let point = CoordPair { y, x };
                let data = layer_distr
                    .sample(map, point, &mut *rng)
                    .map_err(Error::LayerDistribution)?;
                layer.set(map, point, data).map_err(Error::Layer)?;
                progress_logger.increment();
                task::yield_now().await;
            }
        }

        progress_logger.set_status("done");
        Ok(())
    }
}
