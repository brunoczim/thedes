use crate::orient::{Coord, ICoord};
use tree::Map as TreeMap;

/// A node on the map, representing an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Node {
    /// X coordinate of this node.
    pub x: ICoord,
    /// Y coordinate of this node.
    pub y: ICoord,
    /// Width of this node.
    pub width: Coord,
    /// Height of this node.
    pub height: Coord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Entry {
    width: Coord,
    height: Coord,
}

impl Entry {
    fn from_node(node: Node) -> Self {
        Self { width: node.width, height: node.height }
    }

    fn to_node(self, x: ICoord, y: ICoord) -> Node {
        Node { x, y, width: self.width, height: self.height }
    }
}

/// A Map of the game, keeping track of object coordinates and size.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Map {
    xy: TreeMap<(ICoord, ICoord), Entry>,
    yx: TreeMap<(ICoord, ICoord), Entry>,
}

impl Map {
    /// Creates a new empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns object data of an object at a given coordinates. Panics if there
    /// is no object there.
    pub fn at(&self, x: ICoord, y: ICoord) -> Node {
        self.try_at(x, y).unwrap()
    }

    /// Returns object data of an object at a given coordinates. Returns None if
    /// there is no object there.
    pub fn try_at(&self, x: ICoord, y: ICoord) -> Option<Node> {
        self.xy.get(&(x, y)).map(|entry| entry.to_node(x, y))
    }

    /// Tries to insert a node and returns whether it could be inserted.
    pub fn insert(&mut self, node: Node) -> bool {
        let vert_pred =
            self.xy.pred_entry(&(node.x, node.y), true).map_or(true, |entry| {
                let &(x, y) = entry.key();
                x < node.x || entry.get().height <= (node.y - y) as Coord
            });

        let vert_succ =
            self.xy.succ_entry(&(node.x, node.y), true).map_or(true, |entry| {
                let &(x, y) = entry.key();
                x > node.x || node.height <= (y - node.y) as Coord
            });

        let horz_pred =
            self.yx.pred_entry(&(node.y, node.x), true).map_or(true, |entry| {
                let &(y, x) = entry.key();
                y < node.y || entry.get().width <= (node.x - x) as Coord
            });

        let horz_succ =
            self.xy.succ_entry(&(node.y, node.x), true).map_or(true, |entry| {
                let &(y, x) = entry.key();
                y > node.y || node.width <= (x - node.x) as Coord
            });

        let success = vert_pred && vert_succ && horz_pred && horz_succ;

        if success {
            self.xy.insert((node.x, node.y), Entry::from_node(node));
            self.yx.insert((node.y, node.x), Entry::from_node(node));
        }

        success
    }

    /// Tries to move a node horizontally and returns whether it could be moved.
    pub fn move_horz(&mut self, x: ICoord, y: ICoord, new_x: ICoord) -> bool {
        let distance = new_x - x;

        self.try_at(x, y).map_or(false, |node| {
            let wrapped = if distance < 0 {
                self.yx.pred(&(y, x), false).map(|(&(_, prev_x), entry)| {
                    new_x >= prev_x && entry.width <= (new_x - prev_x) as Coord
                })
            } else {
                self.yx.succ(&(y, x), false).map(|(&(_, next_x), _)| {
                    new_x <= next_x && node.width <= (next_x - new_x) as Coord
                })
            };

            let success = wrapped.unwrap_or(true);

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
    pub fn move_vert(&mut self, x: ICoord, y: ICoord, new_y: ICoord) -> bool {
        let distance = new_y - y;

        self.try_at(x, y).map_or(false, |node| {
            let wrapped = if distance < 0 {
                self.xy.pred(&(x, y), false).map(|(&(_, prev_y), entry)| {
                    new_y >= prev_y && entry.height <= (new_y - prev_y) as Coord
                })
            } else {
                self.xy.succ(&(x, y), false).map(|(&(_, next_y), _)| {
                    new_y <= next_y && node.height <= (next_y - new_y) as Coord
                })
            };

            let success = wrapped.unwrap_or(true);

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
        let vert_pred = self.xy.pred(&(new_node.x, new_node.y), false).map_or(
            true,
            |(&(x, y), &entry)| {
                x < new_node.x || entry.height <= (new_node.y - y) as Coord
            },
        );

        let vert_succ = self.xy.succ(&(new_node.x, new_node.y), false).map_or(
            true,
            |(&(x, y), _)| {
                x > new_node.x || new_node.height <= (y - new_node.y) as Coord
            },
        );

        let horz_pred = self.yx.pred(&(new_node.y, new_node.x), false).map_or(
            true,
            |(&(y, x), &entry)| {
                y < new_node.y || entry.width <= (new_node.x - x) as Coord
            },
        );

        let horz_succ = self.xy.succ(&(new_node.y, new_node.x), false).map_or(
            true,
            |(&(y, x), _)| {
                y > new_node.y || new_node.width <= (x - new_node.x) as Coord
            },
        );

        let mut success = vert_pred && vert_succ && horz_pred && horz_succ;

        if success {
            success &= self
                .xy
                .get_mut(&(new_node.x, new_node.y))
                .map(|entry| *entry = Entry::from_node(new_node))
                .is_some();
            success &= self
                .yx
                .get_mut(&(new_node.y, new_node.y))
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
}
