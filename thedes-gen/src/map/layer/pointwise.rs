use std::convert::Infallible;

use thedes_domain::{
    geometry::{CoordPair, Rect},
    map::Map,
};
use thedes_tui::{
    component::task::{ProgressMetric, TaskProgress, TaskReset, TaskTick},
    Tick,
};
use thiserror::Error;

use crate::{
    random::PickedReproducibleRng,
    sm::{Resources, State, StateMachine},
};

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
    pub layer_distr: &'a Ld,
    pub map: &'m mut Map,
    pub rng: &'r mut PickedReproducibleRng,
}

#[derive(Debug, Clone)]
struct GeneratorResources {
    curr: CoordPair,
    progress_goal: ProgressMetric,
    current_progress: ProgressMetric,
}

impl<'a, 'm, 'r, L, Ld> Resources<GeneratorTickArgs<'a, 'm, 'r, L, Ld>>
    for GeneratorResources
where
    L: Layer,
    L::Error: std::error::Error,
    Ld: LayerDistribution<Data = L::Data>,
    Ld::Error: std::error::Error,
{
    type State = GeneratorState;
    type Error = GenError<L::Error, Ld::Error>;

    fn transition(
        &mut self,
        state: Self::State,
        tick: &mut thedes_tui::Tick,
        args: GeneratorTickArgs<'a, 'm, 'r, L, Ld>,
    ) -> Result<Self::State, Self::Error> {
        match state {
            GeneratorState::Init => self.init(tick, args),
            GeneratorState::Done => self.done(tick, args),
            GeneratorState::GeneratingPoint => {
                self.generating_point(tick, args)
            },
        }
    }
}

impl GeneratorResources {
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
        self.fit_progress_goal(args.map.rect());
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
            .layer_distr
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

    fn fit_progress_goal(&mut self, map_rect: Rect) {
        let points = ProgressMetric::from(map_rect.size.y)
            * ProgressMetric::from(map_rect.size.x);
        self.progress_goal = points;
    }
}

impl TaskReset<()> for GeneratorResources {
    type Output = ();
    type Error = Infallible;

    fn reset(&mut self, _args: ()) -> Result<Self::Output, Self::Error> {
        self.curr = CoordPair::default();
        self.current_progress = 0;
        self.progress_goal = 1;
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum GeneratorState {
    Init,
    GeneratingPoint,
    Done,
}

impl State for GeneratorState {
    fn default_initial() -> Self {
        Self::Init
    }

    fn is_final(&self) -> bool {
        matches!(self, Self::Done)
    }
}

#[derive(Debug, Clone)]
pub struct Generator {
    machine: StateMachine<GeneratorResources, GeneratorState>,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            machine: StateMachine::new(GeneratorResources {
                curr: CoordPair::default(),
                progress_goal: 0,
                current_progress: 1,
            }),
        }
    }

    pub fn fit_progress_goal(&mut self, map_rect: Rect) {
        self.machine.resources_mut().fit_progress_goal(map_rect);
    }
}

impl TaskReset<()> for Generator {
    type Output = ();
    type Error = Infallible;

    fn reset(&mut self, args: ()) -> Result<Self::Output, Self::Error> {
        self.machine.reset(args)?;
        Ok(())
    }
}

impl TaskProgress for Generator {
    fn current_progress(&self) -> ProgressMetric {
        self.machine.resources().current_progress
    }

    fn progress_goal(&self) -> ProgressMetric {
        self.machine.resources().progress_goal
    }

    fn progress_status(&self) -> String {
        let state = match self.machine.state() {
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
        self.machine.on_tick(tick, args)
    }
}
