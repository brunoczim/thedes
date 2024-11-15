use thedes_domain::geometry::CoordPair;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmallNode<T> {
    location: CoordPair,
    data: T,
}

impl<T> SmallNode<T> {
    fn new(location: CoordPair, data: T) -> Self {
        Self { location, data }
    }

    pub fn location(&self) -> CoordPair {
        self.location
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

#[derive(Debug, Clone)]
pub struct SmallGraph<T> {
    nodes: Vec<SmallNode<T>>,
    edges: Vec<(CoordPair, CoordPair)>,
}

impl<T> Default for SmallGraph<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SmallGraph<T> {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new() }
    }

    pub fn with_capacity(node_cap: usize, max_edge_per_node: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(node_cap),
            edges: Vec::with_capacity(max_edge_per_node * node_cap / 2),
        }
    }

    pub fn reserve(
        &mut self,
        additional_nodes: usize,
        additional_edges_per_node: usize,
    ) {
        self.nodes.reserve(additional_nodes);
        self.edges.reserve(additional_nodes * additional_edges_per_node);
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }

    pub fn node(&self, location: CoordPair) -> Option<&SmallNode<T>> {
        self.nodes.iter().find(|node| node.location == location)
    }

    fn node_mut(&mut self, location: CoordPair) -> Option<&mut SmallNode<T>> {
        self.nodes.iter_mut().find(|node| node.location == location)
    }

    pub fn node_data(&self, location: CoordPair) -> Option<&T> {
        self.node(location).map(SmallNode::data)
    }

    pub fn node_data_mut(&mut self, location: CoordPair) -> Option<&mut T> {
        self.node_mut(location).map(SmallNode::data_mut)
    }

    pub fn contains_node(&self, location: CoordPair) -> bool {
        self.node(location).is_some()
    }

    pub fn insert_node(&mut self, location: CoordPair, data: T) {
        match self.node_mut(location) {
            Some(node) => {
                *node.data_mut() = data;
            },
            None => {
                let node = SmallNode::new(location, data);
                self.nodes.push(node);
            },
        }
    }

    pub fn remove_node(&mut self, location: CoordPair) -> Option<T> {
        self.disconnect_node(location);
        self.nodes
            .iter()
            .position(|node| node.location == location)
            .map(|index| self.nodes.remove(index).data)
    }

    fn normalize_edge(
        &self,
        node_a: CoordPair,
        node_b: CoordPair,
    ) -> (CoordPair, CoordPair) {
        (node_a.min(node_b), node_a.max(node_b))
    }

    pub fn connected(&self, node_a: CoordPair, node_b: CoordPair) -> bool {
        let edge = self.normalize_edge(node_a, node_b);
        self.edges.iter().any(|elem| *elem == edge)
    }

    pub fn connect(&mut self, node_a: CoordPair, node_b: CoordPair) -> bool {
        let connected = self.connected(node_a, node_b);
        if !connected {
            let edge = self.normalize_edge(node_a, node_b);
            self.edges.push(edge);
        }
        connected
    }

    pub fn disconnect(&mut self, node_a: CoordPair, node_b: CoordPair) -> bool {
        let (normalized_a, normalized_b) = self.normalize_edge(node_a, node_b);
        let count = self.remove_edges(|edge_a, edge_b| {
            normalized_a == edge_a && normalized_b == edge_b
        });
        count > 0
    }

    pub fn disconnect_node(&mut self, node: CoordPair) -> usize {
        self.remove_edges(|node_a, node_b| node_a == node || node_b == node)
    }

    pub fn remove_edges<F>(&mut self, mut filter_out: F) -> usize
    where
        F: FnMut(CoordPair, CoordPair) -> bool,
    {
        let mut disconnected = 0;
        self.edges.retain(|(node_a, node_b)| {
            let filtered_out = filter_out(*node_a, *node_b);
            if filtered_out {
                disconnected += 1
            }
            !filtered_out
        });
        disconnected
    }

    pub fn nodes(&self) -> impl Iterator<Item = &'_ SmallNode<T>> + '_ {
        self.nodes.iter()
    }

    pub fn nodes_data_mut(
        &mut self,
    ) -> impl Iterator<Item = (CoordPair, &'_ mut T)> + '_ {
        self.nodes.iter_mut().map(|node| (node.location(), node.data_mut()))
    }

    pub fn edges(
        &self,
    ) -> impl Iterator<Item = (CoordPair, CoordPair)> + Send + Sync + '_ {
        self.edges.iter().copied()
    }

    pub fn node_edges(
        &self,
        location: CoordPair,
    ) -> impl Iterator<Item = CoordPair> + Send + Sync + '_ {
        self.edges().filter_map(move |(node_a, node_b)| {
            if node_a == location {
                Some(node_b)
            } else if node_b == location {
                Some(node_a)
            } else {
                None
            }
        })
    }
}
