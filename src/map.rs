use crate::orient::{Axis, Coord, Coord2D, Rect};
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
            self.xy.pred_entry(&node.start, true).map_or(true, |entry| {
                let start = *entry.key();
                let size = *entry.get();
                !Rect { start, size }.overlaps(node)
            });

        let vert_succ =
            self.xy.succ_entry(&node.start, true).map_or(true, |entry| {
                let start = *entry.key();
                let size = *entry.get();
                !node.overlaps(Rect { start, size })
            });

        let horz_pred =
            self.yx.pred_entry(&node.start.inv(), true).map_or(true, |entry| {
                let start = entry.key().inv();
                let size = *entry.get();
                !Rect { start, size }.overlaps(node)
            });

        let horz_succ =
            self.yx.succ_entry(&node.start.inv(), true).map_or(true, |entry| {
                let start = entry.key().inv();
                let size = *entry.get();
                !node.overlaps(Rect { start, size })
            });

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
            let neighbour = if old.x > new_x {
                self.yx.pred(&node.start.inv(), false)
            } else {
                self.yx.succ(&node.start.inv(), false)
            };

            let success = neighbour.map_or(true, |(&start, &size)| {
                let start = start.inv();
                !new_node.moves_through(Rect { start, size }, old.x, Axis::X)
            });

            if success {
                self.xy.remove(&old);
                self.yx.remove(&old.inv());
                self.xy.insert(new_node.start, new_node.size);
                self.yx.insert(new_node.start.inv(), new_node.size);
            }

            success
        })
    }

    /// Tries to move a node vertically and returns whether it could be moved.
    pub fn move_vert(&mut self, old: Coord2D, new_y: Coord) -> bool {
        self.try_at(old).map_or(false, |node| {
            let new_node = Rect { start: Coord2D { y: new_y, ..old }, ..node };
            let neighbour = if old.y > new_y {
                self.xy.pred(&node.start, false)
            } else {
                self.xy.succ(&node.start, false)
            };

            let success = neighbour.map_or(true, |(&start, &size)| {
                !new_node.moves_through(Rect { start, size }, old.y, Axis::Y)
            });

            if success {
                self.xy.remove(&old);
                self.yx.remove(&old.inv());
                self.xy.insert(new_node.start, new_node.size);
                self.yx.insert(new_node.start.inv(), new_node.size);
            }

            success
        })
    }

    /// Tries to resize a node and returns whether it could be resized.
    pub fn resize(&mut self, new_node: Rect) -> bool {
        let vert_pred =
            self.xy.pred_entry(&new_node.start, false).map_or(true, |entry| {
                let start = *entry.key();
                let size = *entry.get();
                !Rect { start, size }.overlaps(new_node)
            });

        let vert_succ =
            self.xy.succ_entry(&new_node.start, false).map_or(true, |entry| {
                let start = *entry.key();
                let size = *entry.get();
                !new_node.overlaps(Rect { start, size })
            });

        let horz_pred = self
            .yx
            .pred_entry(&new_node.start.inv(), false)
            .map_or(true, |entry| {
                let start = entry.key().inv();
                let size = *entry.get();
                !Rect { start, size }.overlaps(new_node)
            });

        let horz_succ = self
            .yx
            .succ_entry(&new_node.start.inv(), false)
            .map_or(true, |entry| {
                let start = entry.key().inv();
                let size = *entry.get();
                !new_node.overlaps(Rect { start, size })
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
                .get_mut(&new_node.start.inv())
                .map(|entry| *entry = new_node.size)
                .is_some();
        }

        success
    }
}

#[cfg(test)]
mod test {
    use super::Map;
    use crate::orient::{Coord2D, Rect};

    #[test]
    fn insert_and_get() {
        let mut map = Map::new();
        let node1 = Rect {
            start: Coord2D { x: 0, y: 2 },
            size: Coord2D { x: 5, y: 5 },
        };
        let node2 = Rect {
            start: Coord2D { x: 20, y: 15 },
            size: Coord2D { x: 6, y: 4 },
        };
        let node3 = Rect {
            start: Coord2D { x: 0, y: 8 },
            size: Coord2D { x: 5, y: 5 },
        };
        let node4 = Rect {
            start: Coord2D { x: 6, y: 2 },
            size: Coord2D { x: 5, y: 7 },
        };

        assert!(map.insert(node1));
        assert_eq!(map.at(node1.start), node1);

        assert!(map.insert(node2));
        assert_eq!(map.at(node2.start), node2);
        assert_eq!(map.at(node1.start), node1);

        assert!(map.insert(node3));
        assert_eq!(map.at(node3.start), node3);

        assert!(map.insert(node4));
        assert_eq!(map.at(node4.start), node4);
    }

    #[test]
    fn insert_fails() {
        let mut map = Map::new();
        let node1 = Rect {
            start: Coord2D { x: 0, y: 2 },
            size: Coord2D { x: 5, y: 5 },
        };
        let node2 = Rect {
            start: Coord2D { x: 2, y: 2 },
            size: Coord2D { x: 6, y: 4 },
        };
        let node3 = Rect {
            start: Coord2D { x: 1, y: 3 },
            size: Coord2D { x: 6, y: 8 },
        };

        assert!(map.insert(node1));
        assert_eq!(map.at(node1.start), node1);

        assert!(!map.insert(node2));
        assert_eq!(map.try_at(node2.start), None);
        assert_eq!(map.at(node1.start), node1);

        assert!(!map.insert(node3));
        assert_eq!(map.try_at(node3.start), None);
    }

    #[test]
    fn moving() {
        let mut map = Map::new();
        let mut node1 = Rect {
            start: Coord2D { x: 0, y: 2 },
            size: Coord2D { x: 5, y: 5 },
        };
        let node2 = Rect {
            start: Coord2D { x: 0, y: 15 },
            size: Coord2D { x: 6, y: 4 },
        };

        assert!(map.insert(node1));
        assert!(map.insert(node2));

        assert!(!map.move_vert(node1.start, 17));
        assert!(!map.move_vert(node1.start, 30));
        assert!(!map.move_vert(node1.start, 12));

        assert!(map.move_vert(node1.start, 0));
        node1.start.y = 0;

        assert!(map.move_horz(node1.start, 20));
        node1.start.x = 20;

        assert!(map.move_vert(node1.start, 15));
        node1.start.y = 15;

        assert!(!map.move_horz(node1.start, 0));
        assert!(!map.move_horz(node1.start, 5));
    }

    #[test]
    fn resizing() {
        let mut map = Map::new();
        let mut node1 = Rect {
            start: Coord2D { x: 0, y: 2 },
            size: Coord2D { x: 5, y: 5 },
        };
        let mut node2 = Rect {
            start: Coord2D { x: 0, y: 15 },
            size: Coord2D { x: 6, y: 4 },
        };

        assert!(map.insert(node1));
        assert!(map.insert(node2));

        node1.size.y = 20;
        assert!(!map.resize(node1));
        node1.size.y = 10;
        assert!(map.resize(node1));

        assert!(map.move_horz(node1.start, 15));
        node1.start.x = 15;
        assert!(map.move_vert(node1.start, 15));
        node1.start.y = 15;

        node2.size.x = 20;
        assert!(!map.resize(node2));
        node2.size.x = 10;
        assert!(map.resize(node2));
    }
}
