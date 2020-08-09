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
use std::collections::{BTreeSet, HashMap};

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

        for point in grown.rows() {
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
                .rows()
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
    /// The minimum size required to generate a house.
    pub min_house_size: Coord2<Nat>,
    /// The maximum size required to generate a house.
    pub max_house_size: Coord2<Nat>,
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
            min_house_size: self.min_house_size,
            max_house_size: self.max_house_size,
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
    min_house_size: Coord2<Nat>,
    max_house_size: Coord2<Nat>,
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
        let mut points = HashMap::new();
        for (vertex_a, vertex_b) in self.village.clone().paths.edges() {
            for axis in Axis::iter() {
                self.collect_sidewalk(&mut points, vertex_a, vertex_b, axis);
            }
        }

        let points = points.into_iter().collect::<Vec<_>>();
        let amount = points.len().min(self.house_attempts as usize);

        for &(point, direc) in points.choose_multiple(&mut self.rng, amount) {
            self.generate_house(point, direc);
        }
    }

    fn collect_sidewalk(
        &mut self,
        points: &mut HashMap<Coord2<Nat>, Direc>,
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
            let mut path_point = vertex_a;
            path_point[axis] = coord;
            self.area.remove(path_point);
            let sidewalk_coords = [
                path_point[!axis].checked_add(1),
                path_point[!axis].checked_sub(1),
            ];

            for coord in sidewalk_coords.iter().filter_map(|&maybe| maybe) {
                let mut sidewalk = path_point;
                sidewalk[!axis] = coord;
                if self.area.contains(sidewalk) {
                    points.insert(
                        sidewalk,
                        sidewalk.straight_direc_to(path_point).unwrap(),
                    );
                }
            }
        }
    }

    fn generate_house(&mut self, door: Coord2<Nat>, direc_to_path: Direc) {
        let limits = self.find_door_limits(door, direc_to_path);
        let actual_max = limits.size.zip_with(self.max_house_size, Ord::min);
        let min_is_possible = self
            .min_house_size
            .zip_with(actual_max, |min, max| min <= max)
            .foldl(|a, b| a & b);

        if min_is_possible {
            let size = self.min_house_size.zip_with(actual_max, |min, max| {
                self.rng.sample(Uniform::new_inclusive(min, max))
            });
            let mut rect = Rect { start: limits.start, size };

            if let Some(diff) = door[direc_to_path.axis()]
                .checked_sub(rect.end()[direc_to_path.axis()])
            {
                rect.start[direc_to_path.axis()] += diff;
            }

            let adjust_min = (door[!direc_to_path.axis()] + 1)
                .saturating_sub(rect.end()[!direc_to_path.axis()]);
            let adjust_max = limits.end()[!direc_to_path.axis()]
                .saturating_sub(rect.end()[!direc_to_path.axis()]);

            rect.start[!direc_to_path.axis()] =
                self.rng.sample(Uniform::new_inclusive(adjust_min, adjust_max));

            if rect.rows().all(|point| self.area.contains(point)) {
                self.insert_house(House { door, rect });
            }
        }
    }

    fn insert_house(&mut self, house: House) {
        self.village.houses.insert(house);
        for point in house.rect.rows() {
            self.area.remove(point);
        }
    }

    fn find_door_limits(
        &self,
        door: Coord2<Nat>,
        direc_to_path: Direc,
    ) -> Rect {
        let limit_cw = self
            .area
            .last_neighbour(door, direc_to_path.rotate_clockwise())
            .unwrap()[!direc_to_path.axis()];
        let limit_ccw = self
            .area
            .last_neighbour(door, direc_to_path.rotate_countercw())
            .unwrap()[!direc_to_path.axis()];

        let limit_behind =
            self.area.last_neighbour(door, !direc_to_path).unwrap()
                [direc_to_path.axis()];
        let limit_front = door[direc_to_path.axis()];

        let (lateral_start, lateral_size) = if limit_cw < limit_ccw {
            (limit_cw, limit_ccw - limit_cw + 1)
        } else {
            (limit_ccw, limit_cw - limit_ccw + 1)
        };

        let (central_start, central_size) = if limit_behind < limit_front {
            (limit_behind, limit_front - limit_behind + 1)
        } else {
            (limit_front, limit_behind - limit_front + 1)
        };

        let mut rect = Rect::default();
        rect.start[direc_to_path.axis()] = central_start;
        rect.start[!direc_to_path.axis()] = lateral_start;
        rect.size[direc_to_path.axis()] = central_size;
        rect.size[!direc_to_path.axis()] = lateral_size;
        rect
    }
}
