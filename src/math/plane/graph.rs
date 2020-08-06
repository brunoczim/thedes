use crate::math::plane::{Coord2, Direc, DirecMap, DirecVector, Nat, Set};
use priority_queue::PriorityQueue;
use std::{cmp, collections::HashMap};

/// The directions that a vertex is connected to.
pub type VertexEdges = DirecMap<bool>;

/// A planar graph of 2d points. Edges can only go in straight lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    edges: HashMap<Coord2<Nat>, VertexEdges>,
    neighbours: Set,
}

impl Graph {
    /// Creates a new empty graph.
    pub fn new() -> Self {
        Graph { edges: HashMap::new(), neighbours: Set::new() }
    }

    /// Returns the underlying set of vertices.
    pub fn as_set(&self) -> &Set {
        &self.neighbours
    }

    /// Returns the directions that a given vertex is connected to.
    pub fn vertex_edges(&self, vertex: Coord2<Nat>) -> Option<VertexEdges> {
        self.edges.get(&vertex).map(Clone::clone)
    }

    /// Tests whether two vertices are connected.
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

        edges[direc]
            && self.as_set().neighbour(vertex_a, direc) == Some(vertex_b)
    }

    /// Inserts a vertex. If the vertex is inside an edge connecting vertices A
    /// and B, the edge will be split in two, connecting A and the new vertex,
    /// and connecting B and the new vertex.
    pub fn insert_vertex(&mut self, vertex: Coord2<Nat>) {
        self.neighbours.insert(vertex);

        let mut edges =
            DirecMap { up: false, left: false, down: false, right: false };

        for direc in Direc::iter() {
            if let Some(neighbour) = self.as_set().neighbour(vertex, direc) {
                let neighbour_edges =
                    self.vertex_edges(neighbour).expect("Inconsistent graph");
                if neighbour_edges[!direc] {
                    edges[direc] = true;
                }
            }
        }

        self.edges.insert(vertex, edges);
    }

    /// Connects two vertices. The vertices must have a straight horizontal or
    /// vertical line between them. They also must be neighbours.
    pub fn connect(
        &mut self,
        vertex_a: Coord2<Nat>,
        vertex_b: Coord2<Nat>,
    ) -> bool {
        let direc = vertex_a
            .straight_direc_to(vertex_b)
            .expect("no straight direction");

        if self.as_set().neighbour(vertex_a, direc) != Some(vertex_b) {
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

    /// Disconnect two vertices.
    pub fn disconnect(
        &mut self,
        vertex_a: Coord2<Nat>,
        vertex_b: Coord2<Nat>,
    ) -> bool {
        let direc = vertex_a
            .straight_direc_to(vertex_b)
            .expect("no straight direction");

        if self.as_set().neighbour(vertex_a, direc) != Some(vertex_b) {
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

    /// Removes a vertex. Possibly preserves edges. Preserved edges will occur
    /// in pairs, and each pair will become a new single edge. For such pair to
    /// be preserved, the edges must form a straight line.
    pub fn remove_vertex(&mut self, vertex: Coord2<Nat>) -> bool {
        let edges = match self.edges.remove(&vertex) {
            Some(edges) => edges,
            None => return false,
        };
        for direc in Direc::iter() {
            if let Some(neighbour) = self.as_set().neighbour(vertex, direc) {
                if !edges[direc] || !edges[!direc] {
                    let neighbour_edges = self
                        .edges
                        .get_mut(&neighbour)
                        .expect("Inconsistent graph");
                    neighbour_edges[direc] = false;
                }
            }
        }

        self.neighbours.remove(vertex);
        true
    }

    /// Removes a vertex without preserving edges.
    pub fn remove_vertex_with_edges(&mut self, vertex: Coord2<Nat>) -> bool {
        let edges = match self.edges.remove(&vertex) {
            Some(edges) => edges,
            None => return false,
        };
        for direc in Direc::iter() {
            if let Some(neighbour) = self.as_set().neighbour(vertex, direc) {
                if edges[direc] {
                    let neighbour_edges = self
                        .edges
                        .get_mut(&neighbour)
                        .expect("Inconsistent graph");
                    neighbour_edges[direc] = false;
                }
            }
        }

        self.neighbours.remove(vertex);
        true
    }

    /// Makes a path between two vertices. If necessary, intermediate vertices
    /// will be created. Edges will also be created in order to make the path
    /// exist. The steps are returned.
    pub fn make_path(
        &mut self,
        start: Coord2<Nat>,
        goal: Coord2<Nat>,
        borders: &Set,
    ) -> Option<Vec<DirecVector<Nat>>> {
        let mut predecessors = HashMap::new();

        let mut travelled = HashMap::new();
        travelled.insert(start, 0);

        let mut estimated = HashMap::new();
        estimated.insert(start, 0);

        let mut points = PriorityQueue::new();
        points.push(start, cmp::Reverse(0));

        while let Some((point, _)) = points.pop() {
            if point == goal {
                return Some(self.assemble_path(start, goal, &predecessors));
            }
            self.eval_neighbours(
                goal,
                borders,
                point,
                &mut predecessors,
                &mut travelled,
                &mut estimated,
                &mut points,
            );
        }

        None
    }

    /// Given the map of predecessors built during the evaluation, creates the
    /// necessary vertices and edges, as well constructs the steps.
    fn assemble_path(
        &mut self,
        start: Coord2<Nat>,
        goal: Coord2<Nat>,
        predecessors: &HashMap<Coord2<Nat>, Coord2<Nat>>,
    ) -> Vec<DirecVector<Nat>> {
        let mut steps = Vec::<DirecVector<Nat>>::new();
        let mut last_vertex = goal;
        let mut current = goal;

        while current != start {
            let prev = *predecessors.get(&current).unwrap();
            let direc = prev.straight_direc_to(current).unwrap();

            match steps.last_mut() {
                Some(step) if step.direc == direc => step.magnitude += 1,
                _ => {
                    if last_vertex != current {
                        self.insert_vertex(current);
                        self.connect(last_vertex, current);
                        last_vertex = current;
                    }
                    steps.push(DirecVector { magnitude: 1, direc });
                },
            }

            if self.as_set().contains(prev) {
                self.connect(last_vertex, prev);
                last_vertex = prev;
            }

            current = prev;
        }

        steps.reverse();
        steps
    }

    /// Finds the cost of possible paths and evaluate the best path.
    fn eval_neighbours(
        &self,
        goal: Coord2<Nat>,
        borders: &Set,
        point: Coord2<Nat>,
        predecessors: &mut HashMap<Coord2<Nat>, Coord2<Nat>>,
        travelled: &mut HashMap<Coord2<Nat>, Nat>,
        estimated: &mut HashMap<Coord2<Nat>, Nat>,
        points: &mut PriorityQueue<Coord2<Nat>, cmp::Reverse<Nat>>,
    ) {
        for direc in Direc::iter() {
            if let Some(neighbour) = point
                .move_by_direc(direc)
                .filter(|&point| !borders.contains(point))
            {
                let attempt = travelled.get(&point).unwrap() + 1;

                if travelled
                    .get(&neighbour)
                    .map_or(true, |&cost| attempt < cost)
                {
                    predecessors.insert(neighbour, point);
                    travelled.insert(neighbour, attempt);
                    let heuristics =
                        neighbour.abs_distance(goal).foldl(0, |a, b| a + b);
                    let estimative = attempt + heuristics;
                    estimated.insert(neighbour, estimative);
                    points.push(neighbour, cmp::Reverse(estimative));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn astar_search() {
        let mut borders = Set::new();
        borders.insert(Coord2 { x: 0, y: 0 });
        borders.insert(Coord2 { x: 1, y: 0 });
        borders.insert(Coord2 { x: 2, y: 0 });
        borders.insert(Coord2 { x: 3, y: 0 });
        borders.insert(Coord2 { x: 4, y: 0 });
        borders.insert(Coord2 { x: 4, y: 1 });
        borders.insert(Coord2 { x: 4, y: 2 });
        borders.insert(Coord2 { x: 4, y: 3 });
        borders.insert(Coord2 { x: 5, y: 3 });
        borders.insert(Coord2 { x: 6, y: 3 });
        borders.insert(Coord2 { x: 7, y: 3 });
        borders.insert(Coord2 { x: 7, y: 2 });
        borders.insert(Coord2 { x: 7, y: 1 });
        borders.insert(Coord2 { x: 7, y: 0 });
        borders.insert(Coord2 { x: 8, y: 0 });
        borders.insert(Coord2 { x: 9, y: 0 });
        borders.insert(Coord2 { x: 10, y: 0 });
        borders.insert(Coord2 { x: 11, y: 0 });
        borders.insert(Coord2 { x: 11, y: 1 });
        borders.insert(Coord2 { x: 11, y: 2 });
        borders.insert(Coord2 { x: 11, y: 3 });
        borders.insert(Coord2 { x: 11, y: 4 });
        borders.insert(Coord2 { x: 11, y: 5 });
        borders.insert(Coord2 { x: 11, y: 6 });
        borders.insert(Coord2 { x: 11, y: 7 });
        borders.insert(Coord2 { x: 10, y: 7 });
        borders.insert(Coord2 { x: 9, y: 7 });
        borders.insert(Coord2 { x: 8, y: 7 });
        borders.insert(Coord2 { x: 7, y: 7 });
        borders.insert(Coord2 { x: 6, y: 7 });
        borders.insert(Coord2 { x: 5, y: 7 });
        borders.insert(Coord2 { x: 4, y: 7 });
        borders.insert(Coord2 { x: 3, y: 7 });
        borders.insert(Coord2 { x: 2, y: 7 });
        borders.insert(Coord2 { x: 1, y: 7 });
        borders.insert(Coord2 { x: 0, y: 7 });
        borders.insert(Coord2 { x: 0, y: 6 });
        borders.insert(Coord2 { x: 0, y: 5 });
        borders.insert(Coord2 { x: 0, y: 4 });
        borders.insert(Coord2 { x: 0, y: 3 });
        borders.insert(Coord2 { x: 0, y: 2 });
        borders.insert(Coord2 { x: 0, y: 1 });

        let start = Coord2 { x: 2, y: 2 };
        let goal = Coord2 { x: 10, y: 2 };
        let mut graph = Graph::new();
        graph.insert_vertex(start);
        graph.insert_vertex(Coord2 { x: 7, y: 7 });
        graph.insert_vertex(goal);

        let path = graph.make_path(start, goal, &borders);

        assert_eq!(
            path.unwrap(),
            &[
                DirecVector { direc: Direc::Right, magnitude: 1 },
                DirecVector { direc: Direc::Down, magnitude: 2 },
                DirecVector { direc: Direc::Right, magnitude: 5 },
                DirecVector { direc: Direc::Up, magnitude: 2 },
                DirecVector { direc: Direc::Right, magnitude: 2 },
            ]
        );
    }
}
