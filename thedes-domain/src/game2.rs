use thedes_ecs::{
    component::{self, Component},
    entity,
    error::ResultMapExt,
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

pub struct Game2Input {
    pub map_rect: Rect,
    pub player_head_pos: CoordPair,
    pub player_head_dir: Direction,
}

#[derive(Debug, Clone)]
pub struct Game2 {
    world: World,
    entities: Entities,
    components: Components,
    systems: Systems,
}

impl Game2 {
    pub fn from_world(mut world: World) -> Result<Self, FromWorldError> {
        let entities = Entities::from_world(&mut world);
        let components = Components::from_world(&mut world);
        let systems = Systems::from_world(&mut world, &components)?;
        Ok(Self { world, entities, components, systems })
    }
}
