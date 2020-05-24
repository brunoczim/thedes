use crate::math::plane::{Coord2, Direc, DirecMap, Nat, Set};
use std::collections::HashMap;

pub type VertexEdges = DirecMap<bool>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    edges: HashMap<Coord2<Nat>, VertexEdges>,
    neighbours: Set,
}

impl Graph {
    pub fn new() -> Self {
        Graph { edges: HashMap::new(), neighbours: Set::new() }
    }

    pub fn as_set(&self) -> &Set {
        &self.neighbours
    }

    pub fn vertex_edges(&self, vertex: Coord2<Nat>) -> Option<VertexEdges> {
        self.edges.get(&vertex).map(Clone::clone)
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

        edges[direc]
            && self.as_set().neighbour(vertex_a, direc) == Some(vertex_b)
    }

    pub fn insert_vertex(&mut self, vertex: Coord2<Nat>) {
        self.neighbours.insert(vertex);

        let mut edges =
            DirecMap { up: false, left: false, down: false, right: false };

        for direc in Direc::iter() {
            if let Some(neighbour) = self.as_set().neighbour(vertex, direc) {
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
}
