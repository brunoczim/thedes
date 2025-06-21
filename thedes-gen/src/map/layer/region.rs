use std::{collections::HashSet, convert::Infallible};

use num::rational::Ratio;
use rand::Rng;
use rand_distr::{Triangular, TriangularError};
use thedes_domain::{
    geometry::{Coord, CoordPair},
    map::Map,
};
use thiserror::Error;

use crate::{
    progress,
    random::{MutableDistribution, PickedReproducibleRng},
};

use super::Layer;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Error creating random distribution for map layer's region count")]
    CountDist(#[source] TriangularError),
}

#[derive(Debug, Error)]
pub enum Error<Le, De, Ce> {
    #[error("Error manipulating map layer")]
    Layer(#[source] Le),
    #[error("Error generating data of a map region")]
    DataDistr(#[source] De),
    #[error("Error collecting regions for outside generation components")]
    Collection(#[source] Ce),
}

#[derive(Debug, Clone, Error)]
pub enum InvalidRegionConfig {
    #[error(
        "Minimum region count ratio {min} cannot be greater than maximum {max}"
    )]
    CountBoundOrder { min: Ratio<Coord>, max: Ratio<Coord> },
    #[error(
        "Peak ratio of region count distribution {peak} must be between min \
         and max rationes {min} and {max}"
    )]
    PeakOutOfBounds { min: Ratio<Coord>, peak: Ratio<Coord>, max: Ratio<Coord> },
    #[error("Range must be in the interval [0, 1], given {ratio}")]
    RatioRange { ratio: Ratio<Coord> },
}

pub trait Collector<T> {
    type Error: std::error::Error;

    fn add_region(
        &mut self,
        center: CoordPair,
        data: &T,
    ) -> Result<(), Self::Error>;

    fn add_point(
        &mut self,
        region: usize,
        point: CoordPair,
    ) -> Result<(), Self::Error>;
}

impl<'a, C, T> Collector<T> for &'a mut C
where
    C: Collector<T> + ?Sized,
{
    type Error = C::Error;

    fn add_region(
        &mut self,
        center: CoordPair,
        data: &T,
    ) -> Result<(), Self::Error> {
        (**self).add_region(center, data)
    }

    fn add_point(
        &mut self,
        region: usize,
        point: CoordPair,
    ) -> Result<(), Self::Error> {
        (**self).add_point(region, point)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NopCollector;

impl<T> Collector<T> for NopCollector {
    type Error = Infallible;

    fn add_region(
        &mut self,
        _center: CoordPair,
        _data: &T,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn add_point(
        &mut self,
        _region: usize,
        _point: CoordPair,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    min_region_count: Ratio<Coord>,
    max_region_count: Ratio<Coord>,
    peak_region_count: Ratio<Coord>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            min_region_count: Ratio::new(1, 30),
            max_region_count: Ratio::new(1, 10),
            peak_region_count: Ratio::new(1, 20),
        }
    }

    pub fn with_min_region_count(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, InvalidRegionConfig> {
        if ratio < Ratio::ZERO || ratio > Ratio::ONE {
            Err(InvalidRegionConfig::RatioRange { ratio })?;
        }
        if ratio > self.max_region_count {
            Err(InvalidRegionConfig::CountBoundOrder {
                min: ratio,
                max: self.max_region_count,
            })?;
        }
        if ratio > self.peak_region_count {
            Err(InvalidRegionConfig::PeakOutOfBounds {
                min: ratio,
                peak: self.peak_region_count,
                max: self.max_region_count,
            })?;
        }
        Ok(Self { min_region_count: ratio, ..self })
    }

    pub fn with_max_region_count(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, InvalidRegionConfig> {
        if ratio < Ratio::ZERO || ratio > Ratio::ONE {
            Err(InvalidRegionConfig::RatioRange { ratio })?;
        }
        if self.min_region_count > ratio {
            Err(InvalidRegionConfig::CountBoundOrder {
                min: self.min_region_count,
                max: ratio,
            })?;
        }
        if self.peak_region_count > ratio {
            Err(InvalidRegionConfig::PeakOutOfBounds {
                min: self.min_region_count,
                peak: self.peak_region_count,
                max: ratio,
            })?;
        }
        Ok(Self { max_region_count: ratio, ..self })
    }

    pub fn with_peak_region_count(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, InvalidRegionConfig> {
        if ratio < Ratio::ZERO || ratio > Ratio::ONE {
            Err(InvalidRegionConfig::RatioRange { ratio })?;
        }
        if self.min_region_count > ratio || ratio > self.max_region_count {
            Err(InvalidRegionConfig::PeakOutOfBounds {
                min: self.min_region_count,
                peak: ratio,
                max: self.max_region_count,
            })?;
        }
        Ok(Self { peak_region_count: ratio, ..self })
    }

    pub fn finish(
        self,
        map: &Map,
        rng: &mut PickedReproducibleRng,
    ) -> Result<Generator, InitError> {
        let unified_size = map.rect().size.x + map.rect().size.y;
        let mut actual_min =
            (self.min_region_count * unified_size).ceil().to_integer();
        let mut actual_peak =
            (self.peak_region_count * unified_size).floor().to_integer();
        let mut actual_max =
            (self.max_region_count * unified_size).floor().to_integer();
        actual_min = actual_min.max(unified_size);
        actual_max = actual_max.min(unified_size);
        actual_min = actual_min.min(actual_max);
        actual_max = actual_min.max(actual_min);
        actual_peak = actual_peak.max(actual_min).min(actual_max);
        let min = f64::from(actual_min);
        let max = f64::from(actual_max) + 1.0 - f64::EPSILON;
        let mode = f64::from(actual_peak);
        let distr =
            Triangular::new(min, max, mode).map_err(InitError::CountDist)?;

        let region_count = rng.sample(&distr) as usize;

        Ok(Generator { region_count, config: self })
    }
}

#[derive(Debug)]
pub struct Generator {
    region_count: usize,
    config: Config,
}

impl Generator {
    pub fn region_count(&self) -> usize {
        self.region_count
    }

    pub fn progress_goal(&self, map: &Map) -> usize {
        let area = map.rect().map(usize::from).total_area();
        let region_count = self.region_count();
        let region_data_prog = region_count;
        let init_avail_points_prog = area;
        let shuf_avail_points_prog = 1;
        let init_centers_prog = 1;
        let convert_avail_points_prog = 1;
        let init_frontiers_prog = region_count;
        let shuf_expand_prog = area - region_count;
        region_data_prog
            + init_avail_points_prog
            + shuf_avail_points_prog
            + init_centers_prog
            + convert_avail_points_prog
            + init_frontiers_prog
            + shuf_expand_prog
    }

    pub async fn execute<L, Dd, C>(
        self,
        layer: &L,
        data_distr: &mut Dd,
        map: &mut Map,
        rng: &mut PickedReproducibleRng,
        collector: &mut C,
        progress_logger: progress::Logger,
    ) -> Result<(), Error<L::Error, Dd::Error, C::Error>>
    where
        L: Layer,
        L::Data: Clone,
        Dd: MutableDistribution<L::Data>,
        C: Collector<L::Data>,
    {
        let area = map.rect().map(usize::from).total_area();

        let mut execution = Execution {
            region_count: self.region_count,
            regions_data: Vec::with_capacity(self.region_count),
            region_centers: Vec::with_capacity(self.region_count),
            available_points_seq: Vec::with_capacity(area),
            available_points: HashSet::with_capacity(area),
            region_frontiers: Vec::with_capacity(self.region_count),
            to_be_processed: Vec::new(),
            config: self.config,
            layer,
            data_distr,
            map,
            rng,
            collector,
            progress_logger,
        };

        execution.generate_region_data()?;

        Ok(())
    }
}

#[derive(Debug)]
struct Execution<'a, D, L, Dd, C> {
    region_count: usize,
    regions_data: Vec<D>,
    region_centers: Vec<CoordPair>,
    available_points_seq: Vec<CoordPair>,
    available_points: HashSet<CoordPair>,
    region_frontiers: Vec<(usize, CoordPair)>,
    to_be_processed: Vec<(usize, CoordPair)>,
    config: Config,
    layer: &'a L,
    data_distr: &'a mut Dd,
    map: &'a mut Map,
    rng: &'a mut PickedReproducibleRng,
    collector: &'a mut C,
    progress_logger: progress::Logger,
}

impl<'a, L, Dd, C> Execution<'a, L::Data, L, Dd, C>
where
    L: Layer,
    L::Data: Clone,
    Dd: MutableDistribution<L::Data>,
    C: Collector<L::Data>,
{
    pub fn generate_region_data(
        &mut self,
    ) -> Result<(), Error<L::Error, Dd::Error, C::Error>> {
        self.progress_logger.set_status("generating region data");
        while self.regions_data.len() < self.region_count {
            let region_data = self
                .data_distr
                .sample_mut(self.rng)
                .map_err(Error::DataDistr)?;
            self.regions_data.push(region_data);
            self.progress_logger.increment();
        }
        Ok(())
    }
}
