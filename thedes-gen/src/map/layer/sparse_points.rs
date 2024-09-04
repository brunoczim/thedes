use std::{convert::Infallible, mem};

use thedes_domain::{geometry::CoordPair, map::Map};
use thedes_tui::{
    component::task::{ProgressMetric, TaskProgress, TaskReset, TaskTick},
    Tick,
};
use thiserror::Error;

use crate::random::PickedReproducibleRng;

use super::{Layer, LayerDistribution};

#[derive(Debug, Error)]
pub enum GenError<L, Ld>
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
pub struct GeneratorTickArgs<'a, 'm, 'r, L, Ld> {
    pub layer: &'a L,
    pub layer_dist: &'a Ld,
    pub map: &'m mut Map,
    pub rng: &'r mut PickedReproducibleRng,
}

#[derive(Debug, Clone)]
struct GeneratorResources {
    curr: CoordPair,
    progress_goal: ProgressMetric,
    current_progress: ProgressMetric,
}

impl GeneratorResources {
    fn transition<L, Ld>(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs<L, Ld>,
        state: GeneratorState,
    ) -> Result<GeneratorState, GenError<L::Error, Ld::Error>>
    where
        L: Layer,
        L::Error: std::error::Error,
        Ld: LayerDistribution<Data = L::Data>,
        Ld::Error: std::error::Error,
    {
        match state {
            GeneratorState::Init => self.init(tick, args),
            GeneratorState::Done => self.done(tick, args),
            GeneratorState::GeneratingPoint => {
                self.generating_point(tick, args)
            },
        }
    }

    fn done<L, Ld>(
        &mut self,
        _tick: &mut Tick,
        _args: GeneratorTickArgs<L, Ld>,
    ) -> Result<GeneratorState, GenError<L::Error, Ld::Error>>
    where
        L: Layer,
        L::Error: std::error::Error,
        Ld: LayerDistribution<Data = L::Data>,
        Ld::Error: std::error::Error,
    {
        self.current_progress = self.progress_goal;
        Ok(GeneratorState::Done)
    }

    fn init<L, Ld>(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs<L, Ld>,
    ) -> Result<GeneratorState, GenError<L::Error, Ld::Error>>
    where
        L: Layer,
        L::Error: std::error::Error,
        Ld: LayerDistribution<Data = L::Data>,
        Ld::Error: std::error::Error,
    {
        let points = ProgressMetric::from(args.map.rect().size.y)
            + ProgressMetric::from(args.map.rect().size.x);
        self.progress_goal = points;
        self.curr = args.map.rect().top_left;
        self.generating_point(tick, args)
    }

    fn generating_point<L, Ld>(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs<L, Ld>,
    ) -> Result<GeneratorState, GenError<L::Error, Ld::Error>>
    where
        L: Layer,
        L::Error: std::error::Error,
        Ld: LayerDistribution<Data = L::Data>,
        Ld::Error: std::error::Error,
    {
        if self
            .curr
            .zip2(args.map.rect().bottom_right())
            .any(|(curr, bot_right)| curr >= bot_right)
        {
            return Ok(GeneratorState::Done);
        }
        self.current_progress += 1;
        let data = args
            .layer_dist
            .sample(args.map, self.curr, args.rng)
            .map_err(GenError::LayerDistribution)?;
        args.layer.set(args.map, self.curr, data).map_err(GenError::Layer)?;
        self.curr.x += 1;
        if args.map.rect().bottom_right().x <= self.curr.x {
            self.curr.x = args.map.rect().top_left.x;
            self.curr.y += 1;
            if args.map.rect().bottom_right().y <= self.curr.y {
                return Ok(GeneratorState::Done);
            }
        }
        Ok(GeneratorState::GeneratingPoint)
    }
}

#[derive(Debug, Clone)]
enum GeneratorState {
    Init,
    GeneratingPoint,
    Done,
}

impl GeneratorState {
    pub const INITIAL: Self = Self::Init;
}

#[derive(Debug, Clone)]
pub struct Generator {
    state: GeneratorState,
    resources: GeneratorResources,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            state: GeneratorState::INITIAL,
            resources: GeneratorResources {
                curr: CoordPair::default(),
                progress_goal: 0,
                current_progress: 1,
            },
        }
    }
}

impl TaskReset<()> for Generator {
    type Output = ();
    type Error = Infallible;

    fn reset(&mut self, _args: ()) -> Result<Self::Output, Self::Error> {
        self.resources.curr = CoordPair::default();
        self.resources.current_progress = 0;
        self.resources.progress_goal = 1;
        self.state = GeneratorState::INITIAL;
        Ok(())
    }
}

impl TaskProgress for Generator {
    fn current_progress(&self) -> ProgressMetric {
        self.resources.current_progress
    }

    fn progress_goal(&self) -> ProgressMetric {
        self.resources.progress_goal
    }

    fn progress_status(&self) -> String {
        let state = match &self.state {
            GeneratorState::Init => "initializing block generator",
            GeneratorState::GeneratingPoint => "generating point block",
            GeneratorState::Done => "done",
        };
        state.to_owned()
    }
}

impl<'a, 'm, 'l, L, Ld> TaskTick<GeneratorTickArgs<'a, 'm, 'l, L, Ld>>
    for Generator
where
    L: Layer,
    L::Error: std::error::Error,
    Ld: LayerDistribution<Data = L::Data>,
    Ld::Error: std::error::Error,
{
    type Output = ();
    type Error = GenError<L::Error, Ld::Error>;

    fn on_tick(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs<'a, 'm, 'l, L, Ld>,
    ) -> Result<Option<Self::Output>, Self::Error> {
        let current_state =
            mem::replace(&mut self.state, GeneratorState::INITIAL);
        self.state = self.resources.transition(tick, args, current_state)?;
        match &self.state {
            GeneratorState::Done => Ok(Some(())),
            _ => Ok(None),
        }
    }
}
