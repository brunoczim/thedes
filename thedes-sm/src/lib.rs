use std::mem;

pub trait State: Sized {
    fn default_initial() -> Self;

    fn is_final(&self) -> bool;
}

pub trait Resources<A> {
    type State: State;
    type Error: std::error::Error;

    fn transition(
        &mut self,
        state: Self::State,
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

    pub fn transition<A>(&mut self, args: A) -> Result<(), R::Error>
    where
        R: Resources<A, State = S>,
    {
        let current = mem::replace(&mut self.state, S::default_initial());
        self.state = self.resources.transition(current, args)?;
        Ok(())
    }
}

impl<R, S> StateMachine<R, S> {
    pub fn resources(&self) -> &R {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut R {
        &mut self.resources
    }

    pub fn state(&self) -> &S {
        &self.state
    }
}
