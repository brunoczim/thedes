use thedes_domain::{
    geometry::{Coord, CoordPair},
    map::{AccessError, Map},
    thede,
};
use thiserror::Error;

use super::{region::Collector, Layer};

pub type ThedeLayerError = AccessError;

#[derive(Debug, Error)]
pub enum InitialLandsError {
    #[error("Invalid region index {index} of {len} regions")]
    InvalidRegion { index: usize, len: usize },
}

#[derive(Debug, Clone)]
pub struct ThedeLayer;

impl Layer for ThedeLayer {
    type Data = Option<thede::Id>;
    type Error = ThedeLayerError;

    fn get(
        &self,
        map: &mut Map,
        point: CoordPair,
    ) -> Result<Self::Data, Self::Error> {
        map.get_thede(point)
    }

    fn set(
        &self,
        map: &mut Map,
        point: CoordPair,
        value: Self::Data,
    ) -> Result<(), Self::Error> {
        map.set_thede(point, value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InitialLand {
    pub id: thede::Id,
    pub location: CoordPair,
    pub area: Coord,
}

#[derive(Debug, Clone, Default)]
pub struct InitialLandsCollector {
    entries: Vec<Option<InitialLand>>,
}

impl InitialLandsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn drain(
        &mut self,
    ) -> impl Iterator<Item = InitialLand> + Send + Sync + '_ {
        self.entries.drain(..).filter_map(|opt| opt)
    }
}

impl Collector<Option<thede::Id>> for InitialLandsCollector {
    type Error = InitialLandsError;

    fn add_region(
        &mut self,
        center: CoordPair,
        data: &Option<thede::Id>,
    ) -> Result<(), Self::Error> {
        self.entries.push(data.map(|id| InitialLand {
            id,
            location: center,
            area: 0,
        }));
        Ok(())
    }

    fn add_point(
        &mut self,
        region: usize,
        _point: CoordPair,
    ) -> Result<(), Self::Error> {
        let len = self.entries.len();
        match self.entries.get_mut(region) {
            Some(Some(entry)) => {
                entry.area += 1;
                Ok(())
            },
            Some(None) => Ok(()),
            None => {
                Err(InitialLandsError::InvalidRegion { index: region, len })
            },
        }
    }
}
