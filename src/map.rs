use crate::orient::{IntPos, NatPos};
use tree::Map as TreeMap;

/// A node on the map, representing an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Node {
    /// X coordinate of this node.
    pub x: IntPos,
    /// Y coordinate of this node.
    pub y: IntPos,
    /// Width of this node.
    pub width: NatPos,
    /// Height of this node.
    pub height: NatPos,
}

impl Node {
    fn overlaps_succ(self, succ: Self) -> bool {
        self.width > (succ.x - self.x) as NatPos
            && self.height > (succ.y - self.y) as NatPos
    }

    fn moves_into_succ(self, succ: Self) -> bool {
        (self.x > succ.x || self.width > (succ.x - self.x) as NatPos)
            && (self.y > succ.y || self.height > (succ.y - self.y) as NatPos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Entry {
    width: NatPos,
    height: NatPos,
}

impl Entry {
    fn from_node(node: Node) -> Self {
        Self { width: node.width, height: node.height }
    }

    fn into_node(self, x: IntPos, y: IntPos) -> Node {
        Node { x, y, width: self.width, height: self.height }
    }
}

/// A Map of the game, keeping track of object coordinates and size.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Map {
    xy: TreeMap<(IntPos, IntPos), Entry>,
    yx: TreeMap<(IntPos, IntPos), Entry>,
}

impl Map {
    /// Creates a new empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns object data of an object at a given coordinates. Panics if there
    /// is no object there.
    pub fn at(&self, x: IntPos, y: IntPos) -> Node {
        self.try_at(x, y).unwrap()
    }

    /// Returns object data of an object at a given coordinates. Returns None if
    /// there is no object there.
    pub fn try_at(&self, x: IntPos, y: IntPos) -> Option<Node> {
        self.xy.get(&(x, y)).map(|entry| entry.into_node(x, y))
    }

    /// Tries to insert a node and returns whether it could be inserted.
    pub fn insert(&mut self, node: Node) -> bool {
        let vert_pred = self.xy.pred_entry(&(node.x, node.y), true).map_or(
            false,
            |entry| {
                let &(x, y) = entry.key();
                entry.get().into_node(x, y).overlaps_succ(node)
            },
        );

        let vert_succ = self.xy.succ_entry(&(node.x, node.y), true).map_or(
            false,
            |entry| {
                let &(x, y) = entry.key();
                node.overlaps_succ(entry.get().into_node(x, y))
            },
        );

        let horz_pred = self.yx.pred_entry(&(node.y, node.x), true).map_or(
            false,
            |entry| {
                let &(y, x) = entry.key();
                entry.get().into_node(x, y).overlaps_succ(node)
            },
        );

        let horz_succ = self.yx.succ_entry(&(node.y, node.x), true).map_or(
            false,
            |entry| {
                let &(y, x) = entry.key();
                node.overlaps_succ(entry.get().into_node(x, y))
            },
        );

        let success = !vert_pred && !vert_succ && !horz_pred && !horz_succ;

        if success {
            self.xy.insert((node.x, node.y), Entry::from_node(node));
            self.yx.insert((node.y, node.x), Entry::from_node(node));
        }

        success
    }

    /// Tries to move a node horizontally and returns whether it could be moved.
    pub fn move_horz(&mut self, x: IntPos, y: IntPos, new_x: IntPos) -> bool {
        let distance = new_x - x;

        self.try_at(x, y).map_or(false, |node| {
            let new_node = Node { x: new_x, y, ..node };
            let overlaps = if distance < 0 {
                self.yx.pred(&(y, x), false).map(
                    |(&(prev_y, prev_x), entry)| {
                        let prev_node = entry.into_node(prev_x, prev_y);
                        prev_node.moves_into_succ(new_node)
                    },
                )
            } else {
                self.yx.succ(&(y, x), false).map(
                    |(&(next_y, next_x), entry)| {
                        let next_node = entry.into_node(next_x, next_y);
                        new_node.moves_into_succ(next_node)
                    },
                )
            };

            let success = !overlaps.unwrap_or(false);

            if success {
                self.yx.remove(&(y, x));
                self.xy.remove(&(x, y));
                self.yx.insert((y, new_x), Entry::from_node(node));
                self.xy.insert((new_x, y), Entry::from_node(node));
            }

            success
        })
    }

    /// Tries to move a node vertically and returns whether it could be moved.
    pub fn move_vert(&mut self, x: IntPos, y: IntPos, new_y: IntPos) -> bool {
        let distance = new_y - y;

        self.try_at(x, y).map_or(false, |node| {
            let new_node = Node { x, y: new_y, ..node };
            let overlaps = if distance < 0 {
                self.xy.pred(&(x, y), false).map(
                    |(&(prev_x, prev_y), entry)| {
                        let prev_node = entry.into_node(prev_x, prev_y);
                        prev_node.moves_into_succ(new_node)
                    },
                )
            } else {
                self.xy.succ(&(x, y), false).map(
                    |(&(next_x, next_y), entry)| {
                        let next_node = entry.into_node(next_x, next_y);
                        new_node.moves_into_succ(next_node)
                    },
                )
            };

            let success = !overlaps.unwrap_or(false);

            if success {
                self.yx.remove(&(y, x));
                self.xy.remove(&(x, y));
                self.yx.insert((new_y, x), Entry::from_node(node));
                self.xy.insert((x, new_y), Entry::from_node(node));
            }

            success
        })
    }

    /// Tries to resize a node and returns whether it could be resized.
    pub fn resize(&mut self, new_node: Node) -> bool {
        let vert_pred = self
            .xy
            .pred_entry(&(new_node.x, new_node.y), false)
            .map_or(false, |entry| {
                let &(x, y) = entry.key();
                entry.get().into_node(x, y).overlaps_succ(new_node)
            });

        let vert_succ = self
            .xy
            .succ_entry(&(new_node.x, new_node.y), false)
            .map_or(false, |entry| {
                let &(x, y) = entry.key();
                new_node.overlaps_succ(entry.get().into_node(x, y))
            });

        let horz_pred = self
            .yx
            .pred_entry(&(new_node.y, new_node.x), false)
            .map_or(false, |entry| {
                let &(y, x) = entry.key();
                entry.get().into_node(x, y).overlaps_succ(new_node)
            });

        let horz_succ = self
            .yx
            .succ_entry(&(new_node.y, new_node.x), false)
            .map_or(false, |entry| {
                let &(y, x) = entry.key();
                new_node.overlaps_succ(entry.get().into_node(x, y))
            });

        let mut success = !vert_pred && !vert_succ && !horz_pred && !horz_succ;

        if success {
            success &= self
                .xy
                .get_mut(&(new_node.x, new_node.y))
                .map(|entry| *entry = Entry::from_node(new_node))
                .is_some();
            success &= self
                .yx
                .get_mut(&(new_node.y, new_node.x))
                .map(|entry| *entry = Entry::from_node(new_node))
                .is_some();
        }

        success
    }
}

#[cfg(test)]
mod test {
    use super::{Map, Node};

    #[test]
    fn insert_and_get() {
        let mut map = Map::new();
        let node1 = Node { x: 0, y: 2, width: 5, height: 5 };
        let node2 = Node { x: 20, y: 15, width: 6, height: 4 };
        let node3 = Node { x: 0, y: 8, width: 5, height: 5 };
        let node4 = Node { x: 0, y: -8, width: 5, height: 7 };
        let node5 = Node { x: -5, y: 2, width: 5, height: 5 };
        let node6 = Node { x: 6, y: 2, width: 5, height: 7 };

        assert!(map.insert(node1));
        assert_eq!(map.at(0, 2), node1);

        assert!(map.insert(node2));
        assert_eq!(map.at(20, 15), node2);
        assert_eq!(map.at(0, 2), node1);

        assert!(map.insert(node3));
        assert_eq!(map.at(0, 8), node3);

        assert!(map.insert(node4));
        assert_eq!(map.at(0, -8), node4);

        assert!(map.insert(node5));
        assert_eq!(map.at(-5, 2), node5);

        assert!(map.insert(node6));
        assert_eq!(map.at(6, 2), node6);
    }

    #[test]
    fn insert_fails() {
        let mut map = Map::new();
        let node1 = Node { x: 0, y: 2, width: 5, height: 5 };
        let node2 = Node { x: 2, y: 2, width: 6, height: 4 };
        let node3 = Node { x: 0, y: -2, width: 6, height: 8 };
        let node4 = Node { x: 1, y: 3, width: 6, height: 8 };

        assert!(map.insert(node1));
        assert_eq!(map.at(0, 2), node1);

        assert!(!map.insert(node2));
        assert_eq!(map.try_at(2, 2), None);
        assert_eq!(map.at(0, 2), node1);

        assert!(!map.insert(node3));
        assert_eq!(map.try_at(0, -2), None);

        assert!(!map.insert(node4));
        assert_eq!(map.try_at(1, 3), None);
    }

    #[test]
    fn moving() {
        let mut map = Map::new();
        let node1 = Node { x: 0, y: 2, width: 5, height: 5 };
        let node2 = Node { x: 0, y: 15, width: 6, height: 4 };

        assert!(map.insert(node1));
        assert!(map.insert(node2));

        assert!(!map.move_vert(0, 2, 17));
        assert!(!map.move_vert(0, 2, 30));
        assert!(!map.move_vert(0, 2, 12));
        assert!(map.move_vert(0, 2, 0));
        assert!(map.move_horz(0, 0, 20));
        assert!(map.move_vert(20, 0, 15));
        assert!(!map.move_horz(20, 15, 0));
        assert!(!map.move_horz(20, 15, 5));
        assert!(!map.move_horz(20, 15, -2));
    }

    #[test]
    fn resizing() {
        let mut map = Map::new();
        let node1 = Node { x: 0, y: 2, width: 5, height: 5 };
        let node2 = Node { x: 0, y: 15, width: 6, height: 4 };

        assert!(map.insert(node1));
        assert!(map.insert(node2));

        assert!(!map.resize(Node { height: 20, ..node1 }));
        assert!(map.resize(Node { height: 10, ..node1 }));

        assert!(map.move_horz(0, 2, -15));
        assert!(map.move_vert(-15, 2, 15));

        let node1 = Node { x: -15, y: 15, ..node1 };

        assert!(!map.resize(Node { width: 20, ..node1 }));
        assert!(map.resize(Node { width: 10, ..node1 }));
    }
}
