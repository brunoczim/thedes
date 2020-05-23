use crate::{
    error::Result,
    math::plane::{Axis, Coord2, Direc, DirecMap, Nat, Rect},
    matter::Block,
    storage::save::SavedGame,
};
use rand::{distributions::weighted::WeightedIndex, Rng};
use std::{
    collections::{BTreeSet, HashMap},
    ops::Bound,
};

pub type PathVertexEdges = DirecMap<bool>;

pub type PathVertexNeighbour = BTreeSet<Coord2<Nat>>;

#[derive(Debug, Clone)]
pub struct PathGraph {
    edges: HashMap<Coord2<Nat>, PathVertexEdges>,
    neighbours: Coord2<PathVertexNeighbour>,
}

impl PathGraph {
    pub fn vertex_edges(&self, vertex: Coord2<Nat>) -> Option<PathVertexEdges> {
        self.edges.get(&vertex).map(Clone::clone)
    }

    pub fn neighbour(
        &self,
        vertex: Coord2<Nat>,
        direc: Direc,
    ) -> Option<Coord2<Nat>> {
        let (axis, start, end) = match direc {
            Direc::Up => (
                Axis::Y,
                Bound::Included(!Coord2 { y: 0, ..vertex }),
                Bound::Excluded(!vertex),
            ),
            Direc::Left => (
                Axis::X,
                Bound::Included(Coord2 { x: 0, ..vertex }),
                Bound::Excluded(vertex),
            ),
            Direc::Down => (
                Axis::Y,
                Bound::Excluded(!vertex),
                Bound::Included(!Coord2 { y: Nat::max_value(), ..vertex }),
            ),
            Direc::Right => (
                Axis::X,
                Bound::Excluded(vertex),
                Bound::Included(Coord2 { x: Nat::max_value(), ..vertex }),
            ),
        };

        self.neighbours[axis].range((start, end)).next().map(Clone::clone)
    }

    pub fn connects(
        &self,
        vertex_a: Coord2<Nat>,
        vertex_b: Coord2<Nat>,
    ) -> bool {
        let direc = match vertex_a.straight_direc_to(vertex_b) {
            Some(direc) => direc,
            None => return false,
        };
        let edges = match self.vertex_edges(vertex_a) {
            Some(edges) => edges,
            None => return false,
        };

        edges[direc] && self.neighbour(vertex_a, direc) == Some(vertex_b)
    }

    pub fn insert_vertex(&mut self, vertex: Coord2<Nat>) {
        self.neighbours.x.insert(vertex);
        self.neighbours.y.insert(!vertex);

        let mut edges =
            DirecMap { up: false, left: false, down: false, right: false };

        for direc in Direc::iter() {
            if let Some(neighbour) = self.neighbour(vertex, direc) {
                let neighbour_edges =
                    self.vertex_edges(neighbour).expect("Inconsitent graph");
                if neighbour_edges[!direc] {
                    edges[direc] = true;
                }
            }
        }

        self.edges.insert(vertex, edges);
    }

    pub fn connect(
        &mut self,
        vertex_a: Coord2<Nat>,
        vertex_b: Coord2<Nat>,
    ) -> bool {
        let direc = vertex_a
            .straight_direc_to(vertex_b)
            .expect("no straight direction");

        if self.neighbour(vertex_a, direc) != Some(vertex_b) {
            panic!("Vertices are not neighbours")
        }

        let edges = self.edges.get_mut(&vertex_a).expect("Invalid vertex");
        if edges[direc] {
            false
        } else {
            edges[direc] = true;
            let edges = self.edges.get_mut(&vertex_b).expect("Invalid vertex");
            edges[!direc] = true;
            true
        }
    }

    pub fn disconnect(
        &mut self,
        vertex_a: Coord2<Nat>,
        vertex_b: Coord2<Nat>,
    ) -> bool {
        let direc = vertex_a
            .straight_direc_to(vertex_b)
            .expect("no straight direction");

        if self.neighbour(vertex_a, direc) != Some(vertex_b) {
            panic!("Vertices are not neighbours")
        }

        let edges = self.edges.get_mut(&vertex_a).expect("Invalid vertex");
        if !edges[direc] {
            false
        } else {
            edges[direc] = false;
            let edges = self.edges.get_mut(&vertex_b).expect("Invalid vertex");
            edges[!direc] = false;
            true
        }
    }

    pub fn remove_vertex(&mut self, vertex: Coord2<Nat>) -> bool {
        let edges = match self.edges.remove(&vertex) {
            Some(edges) => edges,
            None => return false,
        };
        for direc in Direc::iter() {
            if let Some(neighbour) = self.neighbour(vertex, direc) {
                if edges[direc] && edges[!direc] {
                    let neighbour_edges = self
                        .edges
                        .get_mut(&neighbour)
                        .expect("Inconsistent graph");
                    neighbour_edges[direc] = false;
                }
            }
        }
        true
    }
}

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
