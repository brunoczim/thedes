use std::{collections::HashSet, convert::Infallible, mem};

use num::rational::Ratio;
use rand::{seq::SliceRandom, Rng};
use rand_distr::{Distribution, Triangular, TriangularError};
use thedes_domain::{
    geometry::{Coord, CoordPair},
    map::Map,
};
use thedes_geometry::axis::Direction;
use thedes_tui::{
    component::task::{ProgressMetric, Task},
    Tick,
};
use thiserror::Error;

use crate::random::PickedReproducibleRng;

use super::Layer;

#[derive(Debug, Error)]
pub enum GenError<E> {
    #[error("Error manipulating map layer")]
    Layer(#[source] E),
    #[error("Error creating random distribution for map layer's region count")]
    CountDist(#[source] TriangularError),
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

    fn gen_region_count<E>(
        &self,
        map_size: CoordPair,
        rng: &mut PickedReproducibleRng,
    ) -> Result<usize, GenError<E>> {
        let unified_size = map_size.x + map_size.y;
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
        let dist =
            Triangular::new(min, max, mode).map_err(GenError::CountDist)?;

        Ok(rng.sample(&dist) as usize)
    }

    pub fn finish<L, Dd>(self, data_dist: Dd) -> Generator<L, L::Data, Dd>
    where
        L: Layer,
        Dd: Distribution<L::Data>,
    {
        Generator {
            progress_goal: 1,
            progress_status: 0,
            region_count: 0,
            regions_data: Vec::new(),
            region_centers: Vec::new(),
            available_points_seq: Vec::new(),
            available_points: HashSet::new(),
            region_frontiers: Vec::new(),
            to_be_processed: Vec::new(),
            data_dist,
            config: self,
            state: GeneratorState::GeneratingRegionCount,
        }
    }
}

#[derive(Debug, Clone)]
enum GeneratorState {
    GeneratingRegionCount,
    GeneratingRegionData,
    InitializingAvailablePoints(CoordPair),
    ShufflingAvailablePoints,
    InitializingCenters,
    ConvertingAvailablePoints,
    InitializingRegionFrontiers(usize),
    ShufflingRegionFrontiers,
    Expanding,
    Done,
}

#[derive(Debug, Clone)]
pub struct Generator<L, D, Dd>
where
    L: Layer<Data = D>,
    Dd: Distribution<D>,
{
    progress_goal: ProgressMetric,
    progress_status: ProgressMetric,
    region_count: usize,
    regions_data: Vec<L::Data>,
    region_centers: Vec<CoordPair>,
    available_points_seq: Vec<CoordPair>,
    available_points: HashSet<CoordPair>,
    region_frontiers: Vec<(usize, CoordPair)>,
    to_be_processed: Vec<(usize, CoordPair)>,
    data_dist: Dd,
    config: Config,
    state: GeneratorState,
}

impl<'a, L, Dd> Generator<L, L::Data, Dd>
where
    L: Layer + 'a,
    L::Data: Clone,
    Dd: Distribution<L::Data>,
{
    #[inline(always)]
    fn expand_point(
        &mut self,
        map: &mut Map,
        layer: &L,
        region: usize,
        point: CoordPair,
    ) -> Result<(), GenError<L::Error>> {
        layer
            .set(map, point, self.regions_data[region].clone())
            .map_err(GenError::Layer)?;
        for direction in Direction::ALL {
            if let Some(new_point) = point
                .checked_move_unit(direction)
                .filter(|new_point| self.available_points.contains(new_point))
            {
                self.region_frontiers.push((region, new_point));
            }
        }
        Ok(())
    }
}

impl<'a, L, Dd> Task<'a> for Generator<L, L::Data, Dd>
where
    L: Layer + 'a,
    L::Data: Clone,
    Dd: Distribution<L::Data>,
{
    type ResetArgs = Config;
    type ResetOutput = ();
    type ResetError = Infallible;
    type TickArgs = (&'a mut Map, &'a L, &'a mut PickedReproducibleRng);
    type TickOutput = ();
    type TickError = GenError<L::Error>;

    #[inline(always)]
    fn progress_goal(&self) -> ProgressMetric {
        self.progress_goal
    }

    #[inline(always)]
    fn progress_status(&self) -> ProgressMetric {
        self.progress_status
    }

    #[inline(always)]
    fn reset(
        &mut self,
        config: Self::ResetArgs,
    ) -> Result<Self::ResetOutput, Self::ResetError> {
        self.progress_status = 0;
        self.progress_goal = 1;
        self.config = config;
        self.region_count = 0;
        self.regions_data.clear();
        self.available_points_seq.clear();
        self.available_points.clear();
        self.region_centers.clear();
        self.region_frontiers.clear();
        self.to_be_processed.clear();
        Ok(())
    }

    #[inline(always)]
    fn on_tick(
        &mut self,
        _tick: &mut Tick,
        (map, layer, rng): &mut Self::TickArgs,
    ) -> Result<Option<Self::TickOutput>, Self::TickError> {
        loop {
            match mem::replace(
                &mut self.state,
                GeneratorState::GeneratingRegionCount,
            ) {
                GeneratorState::Done => {
                    self.progress_status = self.progress_goal;
                },
                GeneratorState::GeneratingRegionCount => {
                    self.region_count =
                        self.config.gen_region_count(map.rect().size, rng)?;
                    self.state = GeneratorState::GeneratingRegionData;
                    let area =
                        map.rect().map(ProgressMetric::from).total_area();
                    let region_count = self.region_count as ProgressMetric;
                    let region_prog = 1;
                    let region_data_prog = region_count;
                    let init_avail_points_prog = area;
                    let shuf_avail_points_prog = 1;
                    let init_centers_prog = 1;
                    let convert_avail_points_prog = 1;
                    let init_frontiers_prog = region_count;
                    let shuf_expand_prog = area - region_count;
                    self.progress_goal = region_prog
                        + region_data_prog
                        + init_avail_points_prog
                        + shuf_avail_points_prog
                        + init_centers_prog
                        + convert_avail_points_prog
                        + init_frontiers_prog
                        + shuf_expand_prog;
                    self.progress_status = 1;
                    break Ok(None);
                },
                GeneratorState::GeneratingRegionData => {
                    if self.regions_data.len() < self.region_count {
                        self.progress_status += 1;
                        self.regions_data.push(rng.sample(&self.data_dist));
                        self.state = GeneratorState::GeneratingRegionData;
                        break Ok(None);
                    }
                    self.state = GeneratorState::InitializingAvailablePoints(
                        map.rect().top_left,
                    );
                },
                GeneratorState::InitializingAvailablePoints(mut current) => {
                    if current.x >= map.rect().bottom_right().x {
                        current.x = map.rect().top_left.x;
                        current.y += 1;
                    }
                    if current.y < map.rect().bottom_right().y {
                        self.available_points_seq.push(current);
                        self.progress_status += 1;
                        current.x += 1;
                        self.state =
                            GeneratorState::InitializingAvailablePoints(
                                current,
                            );
                        break Ok(None);
                    }
                    self.state = GeneratorState::ShufflingAvailablePoints;
                },
                GeneratorState::ShufflingAvailablePoints => {
                    self.progress_status += 1;
                    self.available_points_seq.shuffle(rng);
                    self.state = GeneratorState::InitializingCenters;
                    break Ok(None);
                },
                GeneratorState::InitializingCenters => {
                    self.progress_status += 1;
                    let drained = self.available_points_seq.drain(
                        self.available_points_seq.len() - self.region_count ..,
                    );
                    self.region_centers.extend(drained);
                    self.state = GeneratorState::ConvertingAvailablePoints;
                    break Ok(None);
                },
                GeneratorState::ConvertingAvailablePoints => {
                    self.progress_status += 1;
                    self.available_points
                        .extend(self.available_points_seq.drain(..));
                    self.state = GeneratorState::InitializingRegionFrontiers(0);
                    break Ok(None);
                },
                GeneratorState::InitializingRegionFrontiers(region) => {
                    if region < self.region_count {
                        self.state =
                            GeneratorState::InitializingRegionFrontiers(
                                region + 1,
                            );
                        self.progress_status += 1;
                        self.expand_point(
                            map,
                            layer,
                            region,
                            self.region_centers[region],
                        )?;
                        break Ok(None);
                    }
                    self.state = GeneratorState::ShufflingRegionFrontiers;
                },
                GeneratorState::ShufflingRegionFrontiers => {
                    if self.available_points.is_empty() {
                        self.progress_status += 1;
                        self.state = GeneratorState::Done;
                        break Ok(Some(()));
                    }
                    self.region_frontiers.shuffle(rng);
                    let process_count =
                        (self.region_frontiers.len() - 1).max(1);
                    let start = self.region_frontiers.len() - process_count;
                    let drained = self.region_frontiers.drain(start ..);
                    self.to_be_processed.extend(drained);
                    self.state = GeneratorState::Expanding;
                    break Ok(None);
                },
                GeneratorState::Expanding => {
                    if let Some((region, point)) = self.to_be_processed.pop() {
                        self.state = GeneratorState::Expanding;
                        if self.available_points.remove(&point) {
                            self.progress_status += 1;
                            self.expand_point(map, layer, region, point)?;
                        }
                        break Ok(None);
                    }
                    self.state = GeneratorState::ShufflingRegionFrontiers;
                    break Ok(None);
                },
            }
        }
    }
}
