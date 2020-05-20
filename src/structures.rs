use crate::{
    error::Result,
    math::plane::{Axis, Coord2, Direc, Nat, Rect},
    matter::Block,
    storage::save::SavedGame,
};
use rand::{distributions::weighted::WeightedIndex, Rng};

/// Rectangular houses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct House {
    /// The rectangle occupied by this house.
    pub rect: Rect,
    /// The door coordinates of this house.
    pub door: Coord2<Nat>,
}

impl House {
    /// Spawns this house into the world.
    pub async fn spawn(self, game: &SavedGame) -> Result<()> {
        for coord in self.rect.borders() {
            if coord != self.door {
                game.map().set_block(coord, Block::Wall).await?;
            }
        }

        Ok(())
    }
}

/// Uniform distribution of rectangle houses.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HouseGenConfig<R>
where
    R: Rng,
{
    pub points: Vec<Coord2<Nat>>,
    /// Minimum rectangle size.
    pub min_size: Coord2<Nat>,
    /// Maximum rectangle size.
    pub max_size: Coord2<Nat>,
    /// Minimum distance between houses.
    pub min_distance: Nat,
    /// Maximum number of attempts, unless minimum houses weren't generated.
    pub attempts: Nat,
    ///  number generator.
    pub rng: R,
}

impl<R> IntoIterator for HouseGenConfig<R>
where
    R: Rng,
{
    type Item = House;
    type IntoIter = HouseGenerator<R>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.points.sort();
        HouseGenerator { config: self }
    }
}

/// The actual generator of houses.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HouseGenerator<R>
where
    R: Rng,
{
    config: HouseGenConfig<R>,
}

impl<R> HouseGenerator<R>
where
    R: Rng,
{
    fn generate_rect(&mut self) -> Rect {
        let point_index =
            self.config.rng.gen_range(0, self.config.points.len());

        let start = self.config.points[point_index];
        let size =
            self.config.min_size.zip_with(self.config.max_size, |min, max| {
                self.config.rng.gen_range(min, max + 1)
            });

        Rect { start, size }
    }

    fn take_points(&mut self, rect: Rect) {
        let grown = Rect {
            start: rect
                .start
                .map(|a| a.saturating_sub(self.config.min_distance)),
            size: rect.size.map(|a| {
                a.saturating_add(self.config.min_distance.saturating_mul(2))
            }),
        };

        for point in grown.lines() {
            if let Ok(pos) = self.config.points.binary_search(&point) {
                self.config.points.remove(pos);
            }
        }
    }

    fn generate_door(&mut self, rect: Rect) -> Coord2<Nat> {
        let weights = [
            (Direc::Up, rect.size.x),
            (Direc::Left, rect.size.y),
            (Direc::Down, rect.size.x),
            (Direc::Right, rect.size.y),
        ];
        let weighted =
            WeightedIndex::new(weights.iter().map(|(_, weight)| weight))
                .expect("Weighted error");
        let index = self.config.rng.sample(weighted);
        let (direc, _) = weights[index];

        let (fixed_axis, limit) = match direc {
            Direc::Up => (Axis::Y, rect.start),
            Direc::Left => (Axis::X, rect.start),
            Direc::Down => (Axis::Y, rect.end().map(|val| val - 1)),
            Direc::Right => (Axis::X, rect.end().map(|val| val - 1)),
        };

        let mut weights = vec![];
        let size = rect.size[!fixed_axis] - 2;
        for i in 0 .. size / 2 {
            weights.push(i)
        }
        for i in size / 2 .. size {
            weights.push(size - i);
        }
        let weighted = WeightedIndex::new(weights).expect("Weighted door");

        let mut door = Coord2 { x: 0, y: 0 };
        door[fixed_axis] = limit[fixed_axis];
        let sampled = self.config.rng.sample(weighted) as Nat;
        door[!fixed_axis] = rect.start[!fixed_axis] + 1 + sampled;

        door
    }
}

impl<R> Iterator for HouseGenerator<R>
where
    R: Rng,
{
    type Item = House;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.config.attempts == 0 || self.config.points.len() == 0 {
                break None;
            }

            self.config.attempts -= 1;

            let rect = self.generate_rect();
            let inside = rect
                .lines()
                .all(|point| self.config.points.binary_search(&point).is_ok());

            if inside {
                self.take_points(rect);

                let door = self.generate_door(rect);
                let house = House { rect, door };
                break Some(house);
            }
        }
    }
}
