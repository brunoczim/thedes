use crate::math::plane::{Coord2, Direc, DirecMap, DirecVector, Nat, Set};
use priority_queue::PriorityQueue;
use std::{
    cmp,
    collections::{hash_map, BTreeSet, HashMap, HashSet},
};

/// The directions that a vertex is connected to.
pub type VertexEdges = DirecMap<bool>;

/// A planar graph of 2d points. Edges can only go in straight lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    edges: HashMap<Coord2<Nat>, VertexEdges>,
    vertices: Set,
}

impl From<Set> for Graph {
    fn from(vertices: Set) -> Self {
        Self::from_vertices(vertices)
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    /// Creates a new empty graph.
    pub fn new() -> Self {
        Self::from_vertices(Set::new())
    }

    /// Creates the graph from the set of vertices. No edges will be created.
    pub fn from_vertices(vertices: Set) -> Self {
        Graph { edges: HashMap::new(), vertices }
    }

    /// Returns the underlying set of vertices.
    pub fn vertices(&self) -> &Set {
        &self.vertices
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
        let direc = match vertex_a.direc_to(vertex_b) {
            Some(direc) => direc,
            None => return false,
        };
        let edges = match self.vertex_edges(vertex_a) {
            Some(edges) => edges,
            None => return false,
        };

        edges[direc]
            && self.vertices().neighbour(vertex_a, direc) == Some(vertex_b)
    }

    /// Inserts a vertex. If the vertex is inside an edge connecting vertices A
    /// and B, the edge will be split in two, connecting A and the new vertex,
    /// and connecting B and the new vertex.
    pub fn insert_vertex(&mut self, vertex: Coord2<Nat>) {
        self.vertices.insert(vertex);

        let mut edges =
            DirecMap { up: false, left: false, down: false, right: false };

        for direc in Direc::iter() {
            if let Some(neighbour) = self.vertices().neighbour(vertex, direc) {
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
        let direc = vertex_a.direc_to(vertex_b).expect("no straight direction");

        if self.vertices().neighbour(vertex_a, direc) != Some(vertex_b) {
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
        let direc = vertex_a.direc_to(vertex_b).expect("no straight direction");

        if self.vertices().neighbour(vertex_a, direc) != Some(vertex_b) {
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
            if let Some(neighbour) = self.vertices().neighbour(vertex, direc) {
                if !edges[direc] || !edges[!direc] {
                    let neighbour_edges = self
                        .edges
                        .get_mut(&neighbour)
                        .expect("Inconsistent graph");
                    neighbour_edges[direc] = false;
                }
            }
        }

        self.vertices.remove(vertex);
        true
    }

    /// Removes a vertex without preserving edges.
    pub fn remove_vertex_with_edges(&mut self, vertex: Coord2<Nat>) -> bool {
        let edges = match self.edges.remove(&vertex) {
            Some(edges) => edges,
            None => return false,
        };
        for direc in Direc::iter() {
            if let Some(neighbour) = self.vertices().neighbour(vertex, direc) {
                if edges[direc] {
                    let neighbour_edges = self
                        .edges
                        .get_mut(&neighbour)
                        .expect("Inconsistent graph");
                    neighbour_edges[direc] = false;
                }
            }
        }

        self.vertices.remove(vertex);
        true
    }

    /// Returns an iterator over the graph's edges. The edges are returned as
    /// pairs of the vertex they connect.
    pub fn edges(&self) -> Edges {
        Edges { graph: self, inner: self.edges.iter(), right: None, down: None }
    }

    /// Returns an iterator of connected components. They can be thought as each
    /// individual "island".
    pub fn components(&self) -> Components {
        Components { graph: self, unvisited: self.vertices().rows().collect() }
    }

    /// Makes a path between two vertices. If necessary, intermediate vertices
    /// will be created. Edges will also be created in order to make the path
    /// exist. The steps are returned.
    pub fn make_path(
        &mut self,
        start: Coord2<Nat>,
        goal: Coord2<Nat>,
        valid_points: &HashSet<Coord2<Nat>>,
    ) -> Option<Vec<DirecVector<Nat>>> {
        let mut predecessors = HashMap::with_capacity(valid_points.len());

        let mut travelled = HashMap::with_capacity(valid_points.len());
        travelled.insert(start, AStarCost { distance: 0, turns: 0 });

        let mut points = PriorityQueue::with_capacity(valid_points.len());
        points.push(start, cmp::Reverse(AStarCost { distance: 0, turns: 0 }));

        while let Some((point, cmp::Reverse(cost))) = points.pop() {
            if point == goal {
                return Some(self.assemble_path(
                    start,
                    goal,
                    &predecessors,
                    cost,
                ));
            }
            self.eval_neighbours(
                goal,
                valid_points,
                point,
                &mut predecessors,
                &mut travelled,
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
        cost: AStarCost,
    ) -> Vec<DirecVector<Nat>> {
        let mut steps = Vec::<DirecVector<Nat>>::with_capacity(
            cost.distance as usize - cost.turns as usize * 2,
        );
        let mut last_vertex = goal;
        let mut current = goal;

        while current != start {
            let prev = *predecessors.get(&current).unwrap();
            let direc = prev.direc_to(current).unwrap();

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

            if self.vertices().contains(prev) {
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
        valid_points: &HashSet<Coord2<Nat>>,
        point: Coord2<Nat>,
        predecessors: &mut HashMap<Coord2<Nat>, Coord2<Nat>>,
        travelled: &mut HashMap<Coord2<Nat>, AStarCost>,
        points: &mut PriorityQueue<Coord2<Nat>, cmp::Reverse<AStarCost>>,
    ) {
        for direc in Direc::iter() {
            if let Some(neighbour) = point
                .move_by_direc(direc)
                .filter(|point| valid_points.contains(point))
            {
                let mut attempt = *travelled.get(&point).unwrap();
                attempt.distance += 1;

                let is_turning = predecessors.get(&point).map(|prev| {
                    prev.direc_to(point) != point.direc_to(neighbour)
                });
                if is_turning.unwrap_or(false) {
                    attempt.turns += 1;
                    // penalty
                    attempt.distance += 2;
                }

                if travelled
                    .get(&neighbour)
                    .map_or(true, |&cost| attempt < cost)
                {
                    predecessors.insert(neighbour, point);
                    travelled.insert(neighbour, attempt);
                    let heuristics =
                        neighbour.abs_distance(goal).fold(|a, b| a + b);
                    attempt.distance += heuristics;
                    points.push(neighbour, cmp::Reverse(attempt));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct AStarCost {
    distance: Nat,
    turns: Nat,
}

/// Iterator over a graph's edges.
#[derive(Debug, Clone)]
pub struct Edges<'graph> {
    graph: &'graph Graph,
    inner: hash_map::Iter<'graph, Coord2<Nat>, VertexEdges>,
    right: Option<Coord2<Nat>>,
    down: Option<Coord2<Nat>>,
}

impl<'graph> Iterator for Edges<'graph> {
    type Item = (Coord2<Nat>, Coord2<Nat>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(right) = self.right.take() {
                break Some((
                    right,
                    self.graph
                        .vertices()
                        .neighbour(right, Direc::Right)
                        .unwrap(),
                ));
            }
            if let Some(down) = self.down.take() {
                break Some((
                    down,
                    self.graph.vertices().neighbour(down, Direc::Down).unwrap(),
                ));
            }
            let (&coord, &map) = self.inner.next()?;
            if map.right {
                self.right = Some(coord);
            }
            if map.down {
                self.down = Some(coord);
            }
        }
    }
}

/// Iterator over a graph's connected components.
#[derive(Debug, Clone)]
pub struct Components<'graph> {
    graph: &'graph Graph,
    unvisited: BTreeSet<Coord2<Nat>>,
}

impl<'graph> Iterator for Components<'graph> {
    type Item = Graph;

    fn next(&mut self) -> Option<Self::Item> {
        let start = *self.unvisited.range(..).next()?;
        let mut stack = vec![start];
        let mut graph = Graph::new();

        graph.insert_vertex(start);
        while let Some(node) = stack.pop() {
            if self.unvisited.remove(&node) {
                for direc in Direc::iter() {
                    if let Some(neighbour) =
                        self.graph.vertices().neighbour(node, direc)
                    {
                        graph.insert_vertex(neighbour);
                        graph.connect(node, neighbour);
                        stack.push(neighbour);
                    }
                }
            }
        }

        Some(graph)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn astar_search() {
        let mut valid_points = HashSet::new();
        for x in 1 ..= 3 {
            for y in 1 ..= 6 {
                valid_points.insert(Coord2 { x, y });
            }
        }
        for x in 4 ..= 7 {
            for y in 4 ..= 6 {
                valid_points.insert(Coord2 { x, y });
            }
        }
        for x in 8 ..= 10 {
            for y in 1 ..= 6 {
                valid_points.insert(Coord2 { x, y });
            }
        }

        let start = Coord2 { x: 2, y: 2 };
        let goal = Coord2 { x: 10, y: 2 };
        let mut graph = Graph::new();
        graph.insert_vertex(start);
        graph.insert_vertex(Coord2 { x: 7, y: 7 });
        graph.insert_vertex(goal);

        let path = graph.make_path(start, goal, &valid_points);

        assert_eq!(
            path.unwrap(),
            &[
                DirecVector { direc: Direc::Down, magnitude: 2 },
                DirecVector { direc: Direc::Right, magnitude: 8 },
                DirecVector { direc: Direc::Up, magnitude: 2 },
            ]
        );
    }

    #[test]
    fn edges() {
        let mut graph = Graph::new();

        graph.insert_vertex(Coord2 { x: 5, y: 5 });
        graph.insert_vertex(Coord2 { x: 7, y: 5 });
        graph.insert_vertex(Coord2 { x: 5, y: 7 });
        graph.connect(Coord2 { x: 5, y: 5 }, Coord2 { x: 7, y: 5 });
        graph.connect(Coord2 { x: 5, y: 5 }, Coord2 { x: 5, y: 7 });

        assert_eq!(
            graph.edges().collect::<Vec<_>>(),
            &[
                (Coord2 { x: 5, y: 5 }, Coord2 { x: 7, y: 5 }),
                (Coord2 { x: 5, y: 5 }, Coord2 { x: 5, y: 7 }),
            ]
        );
    }
}
