use std::mem;

use thedes_tui::component::task::{
    ProgressMetric,
    TaskProgress,
    TaskReset,
    TaskTick,
};

pub trait State: Sized {
    fn default_initial() -> Self;

    fn is_final(&self) -> bool;
}

pub trait Resources<A> {
    type State: State;
    type Error;

    fn transition(
        &mut self,
        state: Self::State,
        tick: &mut thedes_tui::Tick,
        args: A,
    ) -> Result<Self::State, Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct StateMachine<R, S> {
    resources: R,
    state: S,
}

impl<R, S> StateMachine<R, S>
where
    S: State,
{
    pub fn new(resources: R) -> Self {
        Self::with_initial_state(resources, S::default_initial())
    }

    pub fn with_initial_state(resources: R, initial_state: S) -> Self {
        Self { resources, state: initial_state }
    }

    pub fn transition<A>(
        &mut self,
        tick: &mut thedes_tui::Tick,
        args: A,
    ) -> Result<(), R::Error>
    where
        R: Resources<A, State = S>,
    {
        let current = mem::replace(&mut self.state, S::default_initial());
        self.state = self.resources.transition(current, tick, args)?;
        Ok(())
    }

    fn reset_state(&mut self) {
        self.reset_state_with(S::default_initial());
    }
}

impl<R, S> StateMachine<R, S> {
    fn reset_state_with(&mut self, new_state: S) {
        self.state = new_state;
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn resources(&self) -> &R {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut R {
        &mut self.resources
    }
}

impl<R, S, A> TaskReset<A> for StateMachine<R, S>
where
    R: TaskReset<A>,
    S: State,
{
    type Error = R::Error;
    type Output = R::Output;

    fn reset(&mut self, args: A) -> Result<Self::Output, Self::Error> {
        self.reset_state();
        self.resources.reset(args)
    }
}

impl<R, S, A> TaskTick<A> for StateMachine<R, S>
where
    R: Resources<A, State = S>,
    S: State,
{
    type Error = R::Error;
    type Output = ();

    fn on_tick(
        &mut self,
        tick: &mut thedes_tui::Tick,
        args: A,
    ) -> Result<Option<Self::Output>, Self::Error> {
        self.transition(tick, args)?;
        if self.state().is_final() {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}
