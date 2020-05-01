use crate::{
    entity::biome,
    error::Result,
    graphics::{Color, SetBg},
    math::plane::{Camera, Coord2, Nat},
    storage::save,
    terminal,
};
use std::fmt;
use tokio::task;

/// A ground block (background color).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Ground {
    /// This block's ground is grass.
    Grass,
    /// This block's ground is sand.
    Sand,
    /// This block's ground is rock.
    Rock,
}

impl fmt::Display for Ground {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(match self {
            Ground::Grass => "grass",
            Ground::Sand => "sand",
            Ground::Rock => "rock",
        })
    }
}

impl Ground {
    /// Renders this ground type on the screen.
    pub fn render(
        &self,
        pos: Coord2<Nat>,
        camera: Camera,
        screen: &mut terminal::Screen,
    ) {
        if let Some(pos) = camera.convert(pos) {
            let bg = match self {
                Ground::Grass => Color::LightGreen,
                Ground::Sand => Color::LightYellow,
                Ground::Rock => Color::DarkYellow,
            };
            screen.set_colors(pos, SetBg { bg });
        }
    }
}

/// A persitent map of ground types.
#[derive(Debug, Clone)]
pub struct Map {
    tree: sled::Tree,
}

impl Map {
    /// Creates a new map given a tree that stores ground types using coordinate
    /// pairs as keys. A seed is provided to create the noise function.
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = task::block_in_place(|| db.open_tree("ground::Map"))?;
        Ok(Self { tree })
    }

    /// Sets a ground type at a given point.
    pub async fn set(&self, point: Coord2<Nat>, value: &Ground) -> Result<()> {
        let point_vec = save::encode(point)?;
        let ground_vec = save::encode(value)?;
        task::block_in_place(|| self.tree.insert(point_vec, ground_vec))?;
        Ok(())
    }

    /// Gets a ground type at a given point.
    pub async fn get(
        &self,
        point: Coord2<Nat>,
        biomes: &biome::Map,
    ) -> Result<Ground> {
        let point_vec = save::encode(point)?;
        let res = task::block_in_place(|| self.tree.get(point_vec));

        match res? {
            Some(bytes) => Ok(save::decode(&bytes)?),
            None => {
                let ground = biomes.get(point).main_ground();
                self.set(point, &ground).await?;
                Ok(ground)
            },
        }
    }
}
