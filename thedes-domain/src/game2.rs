use thedes_ecs::{
    component::{self, Component},
    entity,
    system,
    world::World,
};
use thedes_geometry::orientation::{Direction, DirectionVec};
use thiserror::Error;

use crate::geometry::{Coord, CoordPair, Rect};

pub const PLAYER: &'static str = "player";
pub const MAP: &'static str = "map";

pub const MOVEMENT: &'static str = "movement";
pub const MOVE_COUNT: &'static str = "move.count";

#[derive(Debug, Error)]
pub enum FromWorldError {
    #[error("failed to manipulate world")]
    Ecs(#[from] thedes_ecs::Error),
}

#[derive(Debug, Error)]
pub enum NewGameError {
    #[error("failed to convert world into game")]
    FromWorld(#[from] FromWorldError),
    #[error("failed to manipulate world")]
    Ecs(#[from] thedes_ecs::Error),
}

#[derive(Debug, Error)]
pub enum MovePlayerError {
    #[error("failed to manipulate world")]
    Ecs(#[from] thedes_ecs::Error),
}

#[derive(Debug, Error)]
pub enum GetPlayerPosError {
    #[error("failed to manipulate world")]
    Ecs(#[from] thedes_ecs::Error),
}

#[derive(Debug, Error)]
pub enum TickError {
    #[error("failed to manipulate world")]
    Ecs(#[from] thedes_ecs::Error),
}

pub struct HeadPos;

impl Component for HeadPos {
    type Value = CoordPair;

    const NAME: &'static str = "head.pos";
}

pub struct Pointer;

impl Component for Pointer {
    type Value = Direction;

    const NAME: &'static str = "pointer";
}

pub struct Speed;

impl Component for Speed {
    type Value = Coord;

    const NAME: &'static str = "speed";
}

pub struct MovesLeft;

impl Component for MovesLeft {
    type Value = Coord;

    const NAME: &'static str = "moves.left";
}

#[derive(Debug, Clone)]
pub struct Entities {
    pub player: entity::Id,
}

impl Entities {
    fn from_world(world: &mut World) -> Self {
        Self { player: world.get_or_create_entity(PLAYER) }
    }
}

#[derive(Debug, Clone)]
pub struct Components {
    pub position: component::Id<HeadPos>,
    pub pointer: component::Id<Pointer>,
    pub speed: component::Id<Speed>,
    pub moves_left: component::Id<MovesLeft>,
}

impl Components {
    fn from_world(world: &mut World) -> Self {
        Self {
            position: world.get_or_create_component(HeadPos),
            pointer: world.get_or_create_component(Pointer),
            speed: world.get_or_create_component(Speed),
            moves_left: world.get_or_create_component(MovesLeft),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Systems {
    movement: system::Id,
    move_count: system::Id,
}

impl Systems {
    fn from_world(
        world: &mut World,
        components: &Components,
    ) -> Result<Self, FromWorldError> {
        Ok(Self {
            movement: world.create_system(
                MOVEMENT,
                (components.position, components.pointer, components.speed),
                |_, (mut position, pointer, speed)| {
                    let Some(new_position) =
                        position.get().checked_move_by(DirectionVec {
                            direction: pointer.try_get()?,
                            magnitude: &speed.get(),
                        })
                    else {
                        return Ok(());
                    };
                    position.set(new_position);
                    Ok(())
                },
            )?,
            move_count: world.create_system(
                MOVE_COUNT,
                (components.speed, components.moves_left),
                |_, (mut speed, mut moves_left)| {
                    let Some(new_moves) = moves_left.get().checked_sub(1)
                    else {
                        return Ok(());
                    };
                    if new_moves == 0 {
                        speed.set(0);
                    }
                    moves_left.set(new_moves);
                    Ok(())
                },
            )?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Game2Input {
    pub player_head_pos: CoordPair,
}

#[derive(Debug, Clone)]
pub struct Game2 {
    world: World,
    entities: Entities,
    components: Components,
    #[expect(unused)]
    systems: Systems,
}

impl Game2 {
    pub fn new(input: Game2Input) -> Result<Self, NewGameError> {
        let world = World::new();
        let mut this = Self::from_world(world)?;
        this.world.set_value(
            this.entities.player,
            this.components.position,
            input.player_head_pos,
        )?;
        Ok(this)
    }

    pub fn from_world(mut world: World) -> Result<Self, FromWorldError> {
        let entities = Entities::from_world(&mut world);
        let components = Components::from_world(&mut world);
        let systems = Systems::from_world(&mut world, &components)?;
        world.create_value(
            entities.player,
            components.pointer,
            Direction::Up,
        )?;
        world.create_value(entities.player, components.speed, 0)?;
        world.create_value(entities.player, components.moves_left, 0)?;
        world.create_value(
            entities.player,
            components.position,
            CoordPair { y: 1000, x: 1000 },
        )?;
        Ok(Self { world, entities, components, systems })
    }

    pub fn move_player(
        &mut self,
        direction: Direction,
    ) -> Result<(), MovePlayerError> {
        self.world.set_value(
            self.entities.player,
            self.components.pointer,
            direction,
        )?;
        self.world.set_value(self.entities.player, self.components.speed, 1)?;
        self.world.set_value(
            self.entities.player,
            self.components.moves_left,
            1,
        )?;
        Ok(())
    }

    pub fn player_pos(&self) -> Result<CoordPair, GetPlayerPosError> {
        let pos = self
            .world
            .get_value(self.entities.player, self.components.position)?;
        Ok(pos)
    }

    pub fn tick(&mut self) -> Result<(), TickError> {
        self.world.tick()?;
        Ok(())
    }
}
