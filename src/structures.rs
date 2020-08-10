use crate::{
    error::Result,
    math::plane::{Axis, Coord2, Direc, Graph, Nat, Rect, Set},
    matter::{Block, Ground},
    storage::save::SavedGame,
};
use rand::{distributions::Uniform, seq::SliceRandom, Rng};
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

#[derive(Debug, Clone)]
pub struct Village {
    pub paths: Graph,
    pub houses: BTreeSet<House>,
    pub debug_doors: Vec<Coord2<Nat>>,
}

impl Village {
    pub async fn spawn(&self, game: &SavedGame) -> Result<()> {
        self.spawn_houses(game).await?;
        self.spawn_path(game).await?;
        Ok(())
    }

    async fn spawn_houses(&self, game: &SavedGame) -> Result<()> {
        for house in &self.houses {
            house.spawn(game).await?;
        }
        for &door in &self.debug_doors {
            game.map().set_ground(door, Ground::DebugDoor).await?;
        }
        Ok(())
    }

    async fn spawn_path(&self, game: &SavedGame) -> Result<()> {
        for (vertex_a, vertex_b) in self.paths.edges() {
            for axis in Axis::iter() {
                let range = if vertex_a[axis] < vertex_b[axis] {
                    vertex_a[axis] ..= vertex_b[axis]
                } else {
                    vertex_b[axis] ..= vertex_a[axis]
                };

                for coord in range {
                    let mut path_point = vertex_a;
                    path_point[axis] = coord;
                    game.map().set_ground(path_point, Ground::Path).await?;
                }
            }
        }

        for vertex in self.paths.vertices().rows() {
            game.map().set_ground(vertex, Ground::DebugVertex).await?;
        }

        Ok(())
    }
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
            village: Village {
                paths: Graph::new(),
                houses: BTreeSet::new(),
                debug_doors: Vec::new(),
            },
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
            if self.area.contains(point) {
                self.generate_house(point, direc);
            }
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
        self.village.debug_doors.push(door);

        let limits = self.find_door_limits(door, direc_to_path);
        let actual_max = limits.size.zip_with(self.max_house_size, Ord::min);
        let min_is_possible = self
            .min_house_size
            .zip_with(actual_max, |min, max| min <= max)
            .fold(|a, b| a & b);

        if min_is_possible {
            let size = self.min_house_size.zip_with(actual_max, |min, max| {
                self.rng.sample(Uniform::new_inclusive(min, max))
            });
            let mut rect = Rect { start: limits.start, size };

            if let Some(diff) = (door[direc_to_path.axis()] + 1)
                .checked_sub(rect.end()[direc_to_path.axis()])
            {
                rect.start[direc_to_path.axis()] += diff;
            }

            let adjust_min = (door[!direc_to_path.axis()] + 2)
                .saturating_sub(rect.end()[!direc_to_path.axis()]);
            let adjust_max = (door[!direc_to_path.axis()] - 1)
                .saturating_sub(rect.start[!direc_to_path.axis()]);

            rect.start[!direc_to_path.axis()] +=
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
