use crate::orient::{Coord, Coord2D, Rect};
use tree::Map as TreeMap;

/// A Map of the game, keeping track of object coordinates and size.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Map {
    xy: TreeMap<Coord2D, Coord2D>,
    yx: TreeMap<Coord2D, Coord2D>,
}

impl Map {
    /// Creates a new empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns object data of an object at a given coordinates. Panics if there
    /// is no object there.
    pub fn at(&self, start: Coord2D) -> Rect {
        self.try_at(start).unwrap()
    }

    /// Returns object data of an object at a given coordinates. Returns None if
    /// there is no object there.
    pub fn try_at(&self, start: Coord2D) -> Option<Rect> {
        self.xy.get(&start).map(|&size| Rect { start, size })
    }

    /// Tries to insert a node and returns whether it could be inserted.
    pub fn insert(&mut self, node: Rect) -> bool {
        let vert_pred =
            self.xy.pred_entry(&node.start, true).map_or(false, |entry| {
                let start = *entry.key();
                let size = *entry.get();
                Rect { start, size }.is_after_in_any(node)
            });

        let vert_succ =
            self.xy.succ_entry(&node.start, true).map_or(false, |entry| {
                let start = *entry.key();
                let size = *entry.get();
                node.is_after_in_any(Rect { start, size })
            });

        let horz_pred = self.yx.pred_entry(&node.start.inv(), true).map_or(
            false,
            |entry| {
                let start = entry.key().inv();
                let size = *entry.get();
                Rect { start, size }.is_after_in_any(node)
            },
        );

        let horz_succ = self.yx.succ_entry(&node.start.inv(), true).map_or(
            false,
            |entry| {
                let start = entry.key().inv();
                let size = *entry.get();
                node.is_after_in_any(Rect { start, size })
            },
        );

        let success = vert_pred && vert_succ && horz_pred && horz_succ;

        if success {
            self.xy.insert(node.start, node.size);
            self.yx.insert(node.start.inv(), node.size);
        }

        success
    }

    /// Tries to move a node horizontally and returns whether it could be moved.
    pub fn move_horz(&mut self, old: Coord2D, new_x: Coord) -> bool {
        self.try_at(old).map_or(false, |node| {
            let new_node = Rect { start: Coord2D { x: new_x, ..old }, ..node };
            let overlaps = if old.x < new_x {
                self.yx.pred(&node.start.inv(), false).map(|(&start, &size)| {
                    let start = start.inv();
                    Rect { start, size }.is_after_in_all(new_node)
                })
            } else {
                self.yx.succ(&node.start.inv(), false).map(|(&start, &size)| {
                    let start = start.inv();
                    new_node.is_after_in_all(Rect { start, size })
                })
            };

            let success = overlaps.unwrap_or(true);

            if success {
                self.xy.remove(&old);
                self.yx.remove(&old.inv());
                self.yx.insert(new_node.start, new_node.size);
                self.xy.insert(new_node.start.inv(), new_node.size);
            }

            success
        })
    }

    /// Tries to move a node vertically and returns whether it could be moved.
    pub fn move_vert(&mut self, old: Coord2D, new_y: Coord) -> bool {
        self.try_at(old).map_or(false, |node| {
            let new_node = Rect { start: Coord2D { y: new_y, ..old }, ..node };
            let overlaps = if old.y < new_y {
                self.xy.pred(&node.start, false).map(|(&start, &size)| {
                    Rect { start, size }.is_after_in_all(new_node)
                })
            } else {
                self.xy.succ(&node.start, false).map(|(&start, &size)| {
                    new_node.is_after_in_all(Rect { start, size })
                })
            };

            let success = overlaps.unwrap_or(true);

            if success {
                self.xy.remove(&old);
                self.yx.remove(&old.inv());
                self.yx.insert(new_node.start, new_node.size);
                self.xy.insert(new_node.start.inv(), new_node.size);
            }

            success
        })
    }

    /// Tries to resize a node and returns whether it could be resized.
    pub fn resize(&mut self, new_node: Rect) -> bool {
        let vert_pred =
            self.xy.pred_entry(&new_node.start, false).map_or(false, |entry| {
                let start = *entry.key();
                let size = *entry.get();
                Rect { start, size }.is_after_in_any(new_node)
            });

        let vert_succ =
            self.xy.succ_entry(&new_node.start, false).map_or(false, |entry| {
                let start = *entry.key();
                let size = *entry.get();
                new_node.is_after_in_any(Rect { start, size })
            });

        let horz_pred = self
            .yx
            .pred_entry(&new_node.start.inv(), false)
            .map_or(false, |entry| {
                let start = entry.key().inv();
                let size = *entry.get();
                Rect { start, size }.is_after_in_any(new_node)
            });

        let horz_succ = self
            .yx
            .succ_entry(&new_node.start.inv(), false)
            .map_or(false, |entry| {
                let start = entry.key().inv();
                let size = *entry.get();
                new_node.is_after_in_any(Rect { start, size })
            });

        let mut success = vert_pred && vert_succ && horz_pred && horz_succ;

        if success {
            success &= self
                .xy
                .get_mut(&new_node.start)
                .map(|entry| *entry = new_node.size)
                .is_some();
            success &= self
                .yx
                .get_mut(&new_node.start)
                .map(|entry| *entry = new_node.size)
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
