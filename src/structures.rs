use crate::{
    error::Result,
    math::plane::{Axis, Coord2, Direc, Graph, Nat, Rect, Set},
    matter::Block,
    storage::save::SavedGame,
};
use rand::{
    distributions::{weighted::WeightedIndex, Uniform},
    seq::SliceRandom,
    Rng,
};
use std::collections::{BTreeSet, HashSet};

/// Rectangular houses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone)]
pub struct Village {
    pub paths: Graph,
    pub houses: BTreeSet<House>,
}

/// Generates path of a thede.
#[derive(Debug, Clone)]
pub struct VillageGenConfig<R>
where
    R: Rng,
{
    /// The (exclusive) borders of the thede.
    pub area: Set,
    /// The minimum attempts to generate vertices.
    pub min_vertex_attempts: Nat,
    /// The maximum attempts to generate vertices.
    pub max_vertex_attempts: Nat,
    /// The minimum attempts to generate extra edges.
    pub min_edge_attempts: Nat,
    /// The maximum attempts to generate extra edges.
    pub max_edge_attempts: Nat,
    /// The minimum attempts to generate houses.
    pub min_house_attempts: Nat,
    /// The maximum attempts to generate houses.
    pub max_house_attempts: Nat,
    /// The random number generator associated with this generator.
    pub rng: R,
}

impl<R> VillageGenConfig<R>
where
    R: Rng,
{
    pub fn gen(mut self) -> Village {
        let vertex_attempts = self.rng.sample(Uniform::new_inclusive(
            self.min_vertex_attempts,
            self.max_vertex_attempts,
        ));
        let edge_attempts = self.rng.sample(Uniform::new_inclusive(
            self.min_edge_attempts,
            self.max_edge_attempts,
        ));
        let house_attempts = self.rng.sample(Uniform::new_inclusive(
            self.min_house_attempts,
            self.max_house_attempts,
        ));
        let mut generator = VillageGen {
            area: self.area,
            village: Village { paths: Graph::new(), houses: BTreeSet::new() },
            vertex_attempts,
            edge_attempts,
            house_attempts,
            rng: self.rng,
        };
        generator.generate();
        generator.village
    }
}

struct VillageGen<R>
where
    R: Rng,
{
    village: Village,
    area: Set,
    vertex_attempts: Nat,
    edge_attempts: Nat,
    house_attempts: Nat,
    rng: R,
}

impl<R> VillageGen<R>
where
    R: Rng,
{
    fn generate(&mut self) {
        self.generate_graph();
        self.generate_houses();
    }

    /// Generates a graph with the paths.
    fn generate_graph(&mut self) {
        self.generate_vertices();
        self.generate_edges();
    }

    /// Generates the vertices of the graph.
    fn generate_vertices(&mut self) {
        let points = self.area.rows().collect::<Vec<_>>();
        let amount = points.len().min(self.vertex_attempts as usize);
        for &point in points.choose_multiple(&mut self.rng, amount) {
            self.village.paths.insert_vertex(point);
        }
    }

    /// Generates the edges of the graph.
    fn generate_edges(&mut self) {
        let mut vertices =
            self.village.paths.vertices().rows().collect::<Vec<_>>();
        vertices.shuffle(&mut self.rng);

        if let Some((&first, rest)) = vertices.split_first() {
            let mut prev = first;
            for &curr in rest {
                self.village.paths.make_path(prev, curr, &self.area);
                prev = curr;
            }
        }

        if vertices.len() >= 2 {
            for _ in 0 .. self.edge_attempts {
                let mut iter = vertices.choose_multiple(&mut self.rng, 2);
                let first = *iter.next().unwrap();
                let second = *iter.next().unwrap();
                self.village.paths.make_path(first, second, &self.area);
            }
        }
    }

    fn generate_houses(&mut self) {
        let mut points = HashSet::new();
        for (vertex_a, vertex_b) in self.village.paths.edges() {
            for axis in Axis::iter() {
                self.collect_sidewalk(&mut points, vertex_a, vertex_b, axis);
            }
        }

        let points = points.into_iter().collect::<Vec<_>>();
        let amount = points.len().min(self.house_attempts as usize);
        let mut area = self.area.clone();

        for point in points.choose_multiple(&mut self.rng, amount) {}
    }

    fn collect_sidewalk(
        &self,
        points: &mut HashSet<Coord2<Nat>>,
        vertex_a: Coord2<Nat>,
        vertex_b: Coord2<Nat>,
        axis: Axis,
    ) {
        let range = if vertex_a[axis] < vertex_b[axis] {
            vertex_a[axis] ..= vertex_b[axis]
        } else {
            vertex_b[axis] ..= vertex_a[axis]
        };

        for coord in range {
            let mut point = vertex_a;
            point[axis] = coord;
            let sidewalk_coords =
                [point[!axis].checked_add(1), point[!axis].checked_sub(1)];

            for coord in sidewalk_coords.iter().filter_map(|&maybe| maybe) {
                let mut sidewalk = point;
                sidewalk[!axis] = coord;
                if self.area.contains(sidewalk) {
                    points.insert(sidewalk);
                }
            }
        }
    }
}
