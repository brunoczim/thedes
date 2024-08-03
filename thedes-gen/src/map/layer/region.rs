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
    component::task::{ProgressMetric, TaskProgress, TaskReset, TaskTick},
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

    pub fn finish<D>(self) -> Generator<D> {
        Generator {
            data: GeneratorData {
                progress_goal: 1,
                current_progress: 0,
                region_count: 0,
                regions_data: Vec::new(),
                region_centers: Vec::new(),
                available_points_seq: Vec::new(),
                available_points: HashSet::new(),
                region_frontiers: Vec::new(),
                to_be_processed: Vec::new(),
                config: self,
            },
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

impl GeneratorState {
    pub const INITIAL: Self = Self::GeneratingRegionCount;
}

#[derive(Debug)]
pub struct GeneratorTickArgs<'a, 'm, 'r, L, Dd> {
    pub layer: &'a L,
    pub data_dist: &'a Dd,
    pub map: &'m mut Map,
    pub rng: &'r mut PickedReproducibleRng,
}

#[derive(Debug, Clone)]
struct GeneratorData<D> {
    progress_goal: ProgressMetric,
    current_progress: ProgressMetric,
    region_count: usize,
    regions_data: Vec<D>,
    region_centers: Vec<CoordPair>,
    available_points_seq: Vec<CoordPair>,
    available_points: HashSet<CoordPair>,
    region_frontiers: Vec<(usize, CoordPair)>,
    to_be_processed: Vec<(usize, CoordPair)>,
    config: Config,
}

impl<D> GeneratorData<D> {
    fn transition<L, Dd>(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd>,
        state: GeneratorState,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        match state {
            GeneratorState::Done => self.done(tick, args),
            GeneratorState::GeneratingRegionCount => {
                self.generating_region_count(tick, args)
            },
            GeneratorState::GeneratingRegionData => {
                self.generating_region_data(tick, args)
            },
            GeneratorState::InitializingAvailablePoints(current) => {
                self.initializing_available_points(tick, args, current)
            },
            GeneratorState::ShufflingAvailablePoints => {
                self.shuffling_available_points(tick, args)
            },
            GeneratorState::InitializingCenters => {
                self.initializing_centers(tick, args)
            },
            GeneratorState::ConvertingAvailablePoints => {
                self.converting_available_points(tick, args)
            },
            GeneratorState::InitializingRegionFrontiers(region) => {
                self.initializing_region_frontiers(tick, args, region)
            },
            GeneratorState::ShufflingRegionFrontiers => {
                self.shuffling_region_frontiers(tick, args)
            },
            GeneratorState::Expanding => self.expanding(tick, args),
        }
    }

    fn done<L, Dd>(
        &mut self,
        _tick: &mut Tick,
        _args: GeneratorTickArgs<L, Dd>,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        self.current_progress = self.progress_goal;
        Ok(GeneratorState::Done)
    }

    fn generating_region_count<L, Dd>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd>,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        self.region_count =
            self.config.gen_region_count(args.map.rect().size, args.rng)?;
        let area = args.map.rect().map(ProgressMetric::from).total_area();
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
        self.current_progress = 1;
        Ok(GeneratorState::GeneratingRegionData)
    }

    fn generating_region_data<L, Dd>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd>,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        if self.regions_data.len() < self.region_count {
            self.current_progress += 1;
            self.regions_data.push(args.rng.sample(args.data_dist));
            Ok(GeneratorState::GeneratingRegionData)
        } else {
            Ok(GeneratorState::InitializingAvailablePoints(
                args.map.rect().top_left,
            ))
        }
    }

    fn initializing_available_points<L, Dd>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd>,
        mut current: CoordPair,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        if current.x >= args.map.rect().bottom_right().x {
            current.x = args.map.rect().top_left.x;
            current.y += 1;
        }
        if current.y < args.map.rect().bottom_right().y {
            self.available_points_seq.push(current);
            self.current_progress += 1;
            current.x += 1;
            Ok(GeneratorState::InitializingAvailablePoints(current))
        } else {
            Ok(GeneratorState::ShufflingAvailablePoints)
        }
    }

    fn shuffling_available_points<L, Dd>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd>,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        self.current_progress += 1;
        self.available_points_seq.shuffle(args.rng);
        Ok(GeneratorState::InitializingCenters)
    }

    fn initializing_centers<L, Dd>(
        &mut self,
        _tick: &mut Tick,
        _args: GeneratorTickArgs<L, Dd>,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        self.current_progress += 1;
        let drained = self
            .available_points_seq
            .drain(self.available_points_seq.len() - self.region_count ..);
        self.region_centers.extend(drained);
        Ok(GeneratorState::ConvertingAvailablePoints)
    }

    fn converting_available_points<L, Dd>(
        &mut self,
        _tick: &mut Tick,
        _args: GeneratorTickArgs<L, Dd>,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        self.current_progress += 1;
        self.available_points.extend(self.available_points_seq.drain(..));
        Ok(GeneratorState::InitializingRegionFrontiers(0))
    }

    fn initializing_region_frontiers<L, Dd>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd>,
        region: usize,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        if region < self.region_count {
            self.current_progress += 1;
            self.expand_point(
                args.map,
                args.layer,
                region,
                self.region_centers[region],
            )?;
            Ok(GeneratorState::InitializingRegionFrontiers(region + 1))
        } else {
            Ok(GeneratorState::ShufflingRegionFrontiers)
        }
    }

    fn shuffling_region_frontiers<L, Dd>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd>,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        if self.available_points.is_empty() {
            self.current_progress += 1;
            Ok(GeneratorState::Done)
        } else {
            self.region_frontiers.shuffle(args.rng);
            let process_count = (self.region_frontiers.len() - 1).max(1);
            let start = self.region_frontiers.len() - process_count;
            let drained = self.region_frontiers.drain(start ..);
            self.to_be_processed.extend(drained);
            Ok(GeneratorState::Expanding)
        }
    }

    fn expanding<L, Dd>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd>,
    ) -> Result<GeneratorState, GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: Distribution<D>,
    {
        if let Some((region, point)) = self.to_be_processed.pop() {
            if self.available_points.remove(&point) {
                self.current_progress += 1;
                self.expand_point(args.map, args.layer, region, point)?;
            }
            Ok(GeneratorState::Expanding)
        } else {
            Ok(GeneratorState::ShufflingRegionFrontiers)
        }
    }

    fn expand_point<L>(
        &mut self,
        map: &mut Map,
        layer: &L,
        region: usize,
        point: CoordPair,
    ) -> Result<(), GenError<L::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
    {
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

#[derive(Debug, Clone)]
pub struct Generator<D> {
    data: GeneratorData<D>,
    state: GeneratorState,
}

impl<D> TaskReset<Config> for Generator<D> {
    type Output = ();
    type Error = Infallible;

    fn reset(&mut self, config: Config) -> Result<Self::Output, Self::Error> {
        self.state = GeneratorState::INITIAL;
        self.data.current_progress = 0;
        self.data.progress_goal = 1;
        self.data.config = config;
        self.data.region_count = 0;
        self.data.regions_data.clear();
        self.data.available_points_seq.clear();
        self.data.available_points.clear();
        self.data.region_centers.clear();
        self.data.region_frontiers.clear();
        self.data.to_be_processed.clear();
        Ok(())
    }
}

impl<D> TaskProgress for Generator<D> {
    fn progress_goal(&self) -> ProgressMetric {
        self.data.progress_goal
    }

    fn current_progress(&self) -> ProgressMetric {
        self.data.current_progress
    }

    fn progress_status(&self) -> String {
        todo!()
    }
}

impl<'a, 'm, 'r, L, Dd> TaskTick<GeneratorTickArgs<'a, 'm, 'r, L, Dd>>
    for Generator<L::Data>
where
    L: Layer,
    L::Data: Clone,
    Dd: Distribution<L::Data>,
{
    type Output = ();
    type Error = GenError<L::Error>;

    fn on_tick(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs<'a, 'm, 'r, L, Dd>,
    ) -> Result<Option<Self::Output>, Self::Error> {
        let current_state =
            mem::replace(&mut self.state, GeneratorState::INITIAL);
        self.state = self.data.transition(tick, args, current_state)?;
        match &self.state {
            GeneratorState::Done => Ok(Some(())),
            _ => Ok(None),
        }
    }
}
