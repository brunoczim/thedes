use std::{collections::HashSet, convert::Infallible};

use num::rational::Ratio;
use rand::{seq::SliceRandom, Rng};
use rand_distr::{Triangular, TriangularError};
use thedes_domain::{
    geometry::{Coord, CoordPair},
    map::Map,
};
use thedes_geometry::orientation::Direction;
use thedes_tui::{
    component::task::{ProgressMetric, TaskProgress, TaskReset, TaskTick},
    Tick,
};
use thiserror::Error;

use crate::{
    random::{MutableDistribution, PickedReproducibleRng},
    sm::{Resources, State, StateMachine},
};

use super::Layer;

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

#[derive(Debug, Error)]
pub enum GenError<Le, De, Ce> {
    #[error("Error manipulating map layer")]
    Layer(#[source] Le),
    #[error("Error generating data of a map region")]
    DataDistr(#[source] De),
    #[error("Error collecting regions for outside generation components")]
    Collection(#[source] Ce),
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

    fn gen_region_count<Le, De, Ce>(
        &self,
        map_size: CoordPair,
        rng: &mut PickedReproducibleRng,
    ) -> Result<usize, GenError<Le, De, Ce>> {
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
        let distr =
            Triangular::new(min, max, mode).map_err(GenError::CountDist)?;

        Ok(rng.sample(&distr) as usize)
    }

    pub fn finish<D>(self) -> Generator<D> {
        Generator {
            machine: StateMachine::new(GeneratorResources {
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
            }),
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

impl State for GeneratorState {
    fn default_initial() -> Self {
        Self::GeneratingRegionCount
    }

    fn is_final(&self) -> bool {
        matches!(self, Self::Done)
    }
}

#[derive(Debug)]
pub struct GeneratorTickArgs<'a, 'd, 'm, 'r, 'c, L, Dd, C> {
    pub layer: &'a L,
    pub data_distr: &'d mut Dd,
    pub map: &'m mut Map,
    pub rng: &'r mut PickedReproducibleRng,
    pub collector: &'c mut C,
}

#[derive(Debug, Clone)]
struct GeneratorResources<D> {
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

impl<D> TaskReset<Config> for GeneratorResources<D> {
    type Output = ();
    type Error = Infallible;

    fn reset(&mut self, config: Config) -> Result<Self::Output, Self::Error> {
        self.current_progress = 0;
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
}

impl<'a, 'd, 'm, 'r, 'c, D, L, Dd, C>
    Resources<GeneratorTickArgs<'a, 'd, 'm, 'r, 'c, L, Dd, C>>
    for GeneratorResources<D>
where
    L: Layer<Data = D>,
    D: Clone,
    Dd: MutableDistribution<D>,
    C: Collector<D>,
{
    type Error = GenError<L::Error, Dd::Error, C::Error>;
    type State = GeneratorState;

    fn transition(
        &mut self,
        state: Self::State,
        tick: &mut thedes_tui::Tick,
        args: GeneratorTickArgs<L, Dd, C>,
    ) -> Result<Self::State, Self::Error> {
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
}

impl<D> GeneratorResources<D> {
    fn done<L, Dd, C, Ce>(
        &mut self,
        _tick: &mut Tick,
        _args: GeneratorTickArgs<L, Dd, C>,
    ) -> Result<GeneratorState, GenError<L::Error, Dd::Error, Ce>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: MutableDistribution<D>,
    {
        self.current_progress = self.progress_goal;
        Ok(GeneratorState::Done)
    }

    fn generating_region_count<L, Dd, C, Ce>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd, C>,
    ) -> Result<GeneratorState, GenError<L::Error, Dd::Error, Ce>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: MutableDistribution<D>,
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

    fn generating_region_data<L, Dd, C, Ce>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd, C>,
    ) -> Result<GeneratorState, GenError<L::Error, Dd::Error, Ce>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: MutableDistribution<D>,
    {
        if self.regions_data.len() < self.region_count {
            self.current_progress += 1;
            let region_data = args
                .data_distr
                .sample_mut(args.rng)
                .map_err(GenError::DataDistr)?;
            self.regions_data.push(region_data);
            Ok(GeneratorState::GeneratingRegionData)
        } else {
            Ok(GeneratorState::InitializingAvailablePoints(
                args.map.rect().top_left,
            ))
        }
    }

    fn initializing_available_points<L, Dd, C, Ce>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd, C>,
        mut current: CoordPair,
    ) -> Result<GeneratorState, GenError<L::Error, Dd::Error, Ce>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: MutableDistribution<D>,
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

    fn shuffling_available_points<L, Dd, C, Ce>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd, C>,
    ) -> Result<GeneratorState, GenError<L::Error, Dd::Error, Ce>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: MutableDistribution<D>,
    {
        self.current_progress += 1;
        self.available_points_seq.shuffle(args.rng);
        Ok(GeneratorState::InitializingCenters)
    }

    fn initializing_centers<L, Dd, C>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd, C>,
    ) -> Result<GeneratorState, GenError<L::Error, Dd::Error, C::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: MutableDistribution<D>,
        C: Collector<D>,
    {
        self.current_progress += 1;
        let drained = self
            .available_points_seq
            .drain(self.available_points_seq.len() - self.region_count ..);
        for (center, data) in drained.zip(&mut self.regions_data) {
            args.collector
                .add_region(center, data)
                .map_err(GenError::Collection)?;
            self.region_centers.push(center);
        }
        Ok(GeneratorState::ConvertingAvailablePoints)
    }

    fn converting_available_points<L, Dd, C, Ce>(
        &mut self,
        _tick: &mut Tick,
        _args: GeneratorTickArgs<L, Dd, C>,
    ) -> Result<GeneratorState, GenError<L::Error, Dd::Error, Ce>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: MutableDistribution<D>,
    {
        self.current_progress += 1;
        self.available_points.extend(self.available_points_seq.drain(..));
        Ok(GeneratorState::InitializingRegionFrontiers(0))
    }

    fn initializing_region_frontiers<L, Dd, C>(
        &mut self,
        _tick: &mut Tick,
        mut args: GeneratorTickArgs<L, Dd, C>,
        region: usize,
    ) -> Result<GeneratorState, GenError<L::Error, Dd::Error, C::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: MutableDistribution<D>,
        C: Collector<D>,
    {
        if region < self.region_count {
            self.current_progress += 1;
            self.expand_point(
                args.map,
                args.layer,
                region,
                self.region_centers[region],
                &mut args.collector,
            )?;
            Ok(GeneratorState::InitializingRegionFrontiers(region + 1))
        } else {
            Ok(GeneratorState::ShufflingRegionFrontiers)
        }
    }

    fn shuffling_region_frontiers<L, Dd, C, Ce>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Dd, C>,
    ) -> Result<GeneratorState, GenError<L::Error, Dd::Error, Ce>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: MutableDistribution<D>,
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

    fn expanding<L, Dd, C>(
        &mut self,
        _tick: &mut Tick,
        mut args: GeneratorTickArgs<L, Dd, C>,
    ) -> Result<GeneratorState, GenError<L::Error, Dd::Error, C::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        Dd: MutableDistribution<D>,
        C: Collector<D>,
    {
        if let Some((region, point)) = self.to_be_processed.pop() {
            if self.available_points.remove(&point) {
                self.current_progress += 1;
                self.expand_point(
                    args.map,
                    args.layer,
                    region,
                    point,
                    &mut args.collector,
                )?;
            }
            Ok(GeneratorState::Expanding)
        } else {
            Ok(GeneratorState::ShufflingRegionFrontiers)
        }
    }

    fn expand_point<L, De, C>(
        &mut self,
        map: &mut Map,
        layer: &L,
        region: usize,
        point: CoordPair,
        collector: &mut C,
    ) -> Result<(), GenError<L::Error, De, C::Error>>
    where
        L: Layer<Data = D>,
        D: Clone,
        C: Collector<D>,
    {
        layer
            .set(map, point, self.regions_data[region].clone())
            .map_err(GenError::Layer)?;
        for direction in Direction::ALL {
            if let Some(new_point) = point
                .checked_move_unit(direction)
                .filter(|new_point| self.available_points.contains(new_point))
            {
                collector
                    .add_point(region, new_point)
                    .map_err(GenError::Collection)?;
                self.region_frontiers.push((region, new_point));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Generator<D> {
    machine: StateMachine<GeneratorResources<D>, GeneratorState>,
}

impl<D> TaskReset<Config> for Generator<D> {
    type Output = ();
    type Error = Infallible;

    fn reset(&mut self, args: Config) -> Result<Self::Output, Self::Error> {
        self.machine.reset(args)
    }
}

impl<D> TaskProgress for Generator<D> {
    fn progress_goal(&self) -> ProgressMetric {
        self.machine.resources().progress_goal
    }

    fn current_progress(&self) -> ProgressMetric {
        self.machine.resources().current_progress
    }

    fn progress_status(&self) -> String {
        let state = match self.machine.state() {
            GeneratorState::Done => "done",
            GeneratorState::GeneratingRegionCount => "generation region count",
            GeneratorState::GeneratingRegionData => "generating region data",
            GeneratorState::InitializingAvailablePoints(_) => {
                "initializing available points"
            },
            GeneratorState::ShufflingAvailablePoints => {
                "shuffling available points"
            },
            GeneratorState::InitializingCenters => {
                "initializing region centers"
            },
            GeneratorState::ConvertingAvailablePoints => {
                "converting available points"
            },
            GeneratorState::InitializingRegionFrontiers(_) => {
                "initializing region frontiers"
            },
            GeneratorState::ShufflingRegionFrontiers
            | GeneratorState::Expanding => "expanding region frontiers",
        };
        state.to_owned()
    }
}

impl<'a, 'd, 'm, 'r, 'c, L, Dd, C>
    TaskTick<GeneratorTickArgs<'a, 'd, 'm, 'r, 'c, L, Dd, C>>
    for Generator<L::Data>
where
    L: Layer,
    L::Data: Clone,
    Dd: MutableDistribution<L::Data>,
    C: Collector<L::Data>,
{
    type Output = ();
    type Error = GenError<L::Error, Dd::Error, C::Error>;

    fn on_tick(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs<'a, 'd, 'm, 'r, 'c, L, Dd, C>,
    ) -> Result<Option<Self::Output>, Self::Error> {
        self.machine.on_tick(tick, args)
    }
}
