use crate::{collections::graph::Node, orientation::Direction, CoordPair};

use super::CoordDiGraph;

#[test]
fn di_contain_node_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 9, x: 3 });
    graph.insert_node(CoordPair { y: 9, x: 21 });
    assert!(graph.contains_node(CoordPair { y: 9, x: 3 }.as_ref()));
}

#[test]
fn di_contain_node_failure() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 9, x: 3 });
    graph.insert_node(CoordPair { y: 9, x: 21 });
    assert!(!graph.contains_node(CoordPair { y: 9, x: 7 }.as_ref()));
}

#[test]
fn neighbors() {
    let mut graph = CoordDiGraph::new();
    for y in 0 .. 6 {
        for x in 0 .. 6 {
            graph.insert_node(CoordPair { y: y * 10, x: x * 10 });
        }
    }

    let neighbors = graph
        .neighbors(CoordPair { y: 30, x: 20 }.as_ref(), Direction::Up)
        .collect::<Vec<_>>();
    assert_eq!(
        neighbors,
        vec![
            (CoordPair { y: 20, x: 20 }.as_ref(), Node::new()),
            (CoordPair { y: 10, x: 20 }.as_ref(), Node::new()),
            (CoordPair { y: 0, x: 20 }.as_ref(), Node::new()),
        ]
    );

    let neighbors = graph
        .neighbors(CoordPair { y: 30, x: 20 }.as_ref(), Direction::Left)
        .collect::<Vec<_>>();
    assert_eq!(
        neighbors,
        vec![
            (CoordPair { y: 30, x: 10 }.as_ref(), Node::new()),
            (CoordPair { y: 30, x: 0 }.as_ref(), Node::new()),
        ]
    );

    let neighbors = graph
        .neighbors(CoordPair { y: 30, x: 20 }.as_ref(), Direction::Down)
        .collect::<Vec<_>>();
    assert_eq!(
        neighbors,
        vec![
            (CoordPair { y: 40, x: 20 }.as_ref(), Node::new()),
            (CoordPair { y: 50, x: 20 }.as_ref(), Node::new()),
        ]
    );

    let neighbors = graph
        .neighbors(CoordPair { y: 30, x: 20 }.as_ref(), Direction::Right)
        .collect::<Vec<_>>();
    assert_eq!(
        neighbors,
        vec![
            (CoordPair { y: 30, x: 30 }.as_ref(), Node::new()),
            (CoordPair { y: 30, x: 40 }.as_ref(), Node::new()),
            (CoordPair { y: 30, x: 50 }.as_ref(), Node::new()),
        ]
    );
}

#[test]
fn neighbors_inclusive() {
    let mut graph = CoordDiGraph::new();
    for y in 0 .. 6 {
        for x in 0 .. 6 {
            graph.insert_node(CoordPair { y: y * 10, x: x * 10 });
        }
    }

    let neighbors = graph
        .neighbors_inclusive(CoordPair { y: 30, x: 20 }.as_ref(), Direction::Up)
        .collect::<Vec<_>>();
    assert_eq!(
        neighbors,
        vec![
            (CoordPair { y: 30, x: 20 }.as_ref(), Node::new()),
            (CoordPair { y: 20, x: 20 }.as_ref(), Node::new()),
            (CoordPair { y: 10, x: 20 }.as_ref(), Node::new()),
            (CoordPair { y: 0, x: 20 }.as_ref(), Node::new()),
        ]
    );

    let neighbors = graph
        .neighbors_inclusive(
            CoordPair { y: 30, x: 20 }.as_ref(),
            Direction::Left,
        )
        .collect::<Vec<_>>();
    assert_eq!(
        neighbors,
        vec![
            (CoordPair { y: 30, x: 20 }.as_ref(), Node::new()),
            (CoordPair { y: 30, x: 10 }.as_ref(), Node::new()),
            (CoordPair { y: 30, x: 0 }.as_ref(), Node::new()),
        ]
    );

    let neighbors = graph
        .neighbors_inclusive(
            CoordPair { y: 30, x: 20 }.as_ref(),
            Direction::Down,
        )
        .collect::<Vec<_>>();
    assert_eq!(
        neighbors,
        vec![
            (CoordPair { y: 30, x: 20 }.as_ref(), Node::new()),
            (CoordPair { y: 40, x: 20 }.as_ref(), Node::new()),
            (CoordPair { y: 50, x: 20 }.as_ref(), Node::new()),
        ]
    );

    let neighbors = graph
        .neighbors_inclusive(
            CoordPair { y: 30, x: 20 }.as_ref(),
            Direction::Right,
        )
        .collect::<Vec<_>>();
    assert_eq!(
        neighbors,
        vec![
            (CoordPair { y: 30, x: 20 }.as_ref(), Node::new()),
            (CoordPair { y: 30, x: 30 }.as_ref(), Node::new()),
            (CoordPair { y: 30, x: 40 }.as_ref(), Node::new()),
            (CoordPair { y: 30, x: 50 }.as_ref(), Node::new()),
        ]
    );
}

#[test]
fn connect_simple_new_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
}

#[test]
fn connect_simple_old_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 41, x: 50 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 41, x: 50 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let is_new = graph
        .connect(CoordPair { y: 41, x: 50 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(!is_new);
}

#[test]
fn connect_intermediate_all_new_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
}

#[test]
fn connect_intermediate_old_e2e_new_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 37, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let is_new = graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
}

#[test]
fn connect_intermediate_all_old_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 37, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let is_new = graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let is_new = graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(!is_new);
}

#[test]
fn connect_bad_dir() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 53 });
    graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 53 })
        .unwrap_err();
}

#[test]
fn connect_unknown() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap_err();
}

#[test]
fn connected_yes() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 37, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let is_connected = graph
        .connected(CoordPair { y: 37, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_connected);
    let is_new = graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let is_connected = graph
        .connected(CoordPair { y: 30, x: 55 }, CoordPair { y: 37, x: 55 })
        .unwrap();
    assert!(is_connected);
    let is_connected = graph
        .connected(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_connected);
}

#[test]
fn connected_yes_only_e2e() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 41, x: 55 }, CoordPair { y: 30, x: 55 })
        .unwrap();
    assert!(is_new);
    let is_connected = graph
        .connected(CoordPair { y: 37, x: 55 }, CoordPair { y: 30, x: 55 })
        .unwrap();
    assert!(is_connected);
    let is_connected = graph
        .connected(CoordPair { y: 41, x: 55 }, CoordPair { y: 30, x: 55 })
        .unwrap();
    assert!(is_connected);
    let is_connected = graph
        .connected(CoordPair { y: 41, x: 55 }, CoordPair { y: 37, x: 55 })
        .unwrap();
    assert!(is_connected);
}

#[test]
fn connected_no() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 37, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let is_connected = graph
        .connected(CoordPair { y: 30, x: 55 }, CoordPair { y: 37, x: 55 })
        .unwrap();
    assert!(!is_connected);
}

#[test]
fn disconnect_simple_effective() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let removed = graph
        .disconnect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(removed);
    let is_connected = graph
        .connected(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(!is_connected);
}

#[test]
fn disconnect_simple_ineffective() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let removed = graph
        .disconnect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(!removed);
    let is_connected = graph
        .connected(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(!is_connected);
}

#[test]
fn disconnect_just_intermediate() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let removed = graph
        .disconnect(CoordPair { y: 37, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(removed);
    let is_connected = graph
        .connected(CoordPair { y: 37, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(!is_connected);
    let is_connected = graph
        .connected(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(!is_connected);
    let is_connected = graph
        .connected(CoordPair { y: 30, x: 55 }, CoordPair { y: 37, x: 55 })
        .unwrap();
    assert!(is_connected);
}

#[test]
fn connect_undirected_simple_new_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect_undirected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 },
        )
        .unwrap();
    assert_eq!(is_new, (true, true));
    assert!(graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
}

#[test]
fn connect_undirected_partial_right_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let is_new = graph
        .connect_undirected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 },
        )
        .unwrap();
    assert_eq!(is_new, (false, true));
    assert!(graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
}

#[test]
fn connect_undirected_partial_left_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    assert!(is_new);
    let is_new = graph
        .connect_undirected(
            CoordPair { y: 41, x: 55 },
            CoordPair { y: 30, x: 55 },
        )
        .unwrap();
    assert_eq!(is_new, (true, false));
    assert!(graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
}

#[test]
fn connect_undirected_intermediate_new_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect_undirected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 },
        )
        .unwrap();
    assert_eq!(is_new, (true, true));
    assert!(graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(graph
        .undirected_connected(
            CoordPair { y: 37, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 37, x: 55 }
        )
        .unwrap());
}

#[test]
fn connect_undirected_mid_partial_left_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    let is_new = graph
        .connect_undirected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 },
        )
        .unwrap();
    assert_eq!(is_new, (false, true));
    assert!(graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(graph
        .undirected_connected(
            CoordPair { y: 37, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 37, x: 55 }
        )
        .unwrap());
}

#[test]
fn disconnect_undirected_intermediate_new_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    let is_new = graph
        .connect_undirected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 },
        )
        .unwrap();
    assert_eq!(is_new, (true, true));
    let removed = graph
        .disconnect_undirected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 },
        )
        .unwrap();
    assert_eq!(removed, (true, true));
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 37, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 37, x: 55 }
        )
        .unwrap());
}

#[test]
fn disconnect_undirected_mid_partial_left_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    let removed = graph
        .disconnect_undirected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 },
        )
        .unwrap();
    assert_eq!(removed, (true, false));
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 37, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 37, x: 55 }
        )
        .unwrap());
}

#[test]
fn disconnect_undirected_mid_partial_right_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    graph
        .connect(CoordPair { y: 30, x: 55 }, CoordPair { y: 41, x: 55 })
        .unwrap();
    let removed = graph
        .disconnect_undirected(
            CoordPair { y: 41, x: 55 },
            CoordPair { y: 30, x: 55 },
        )
        .unwrap();
    assert_eq!(removed, (false, true));
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 37, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 37, x: 55 }
        )
        .unwrap());
}

#[test]
fn disconnect_undirected_sub_success() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    graph
        .connect_undirected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 },
        )
        .unwrap();
    graph
        .disconnect_undirected(
            CoordPair { y: 37, x: 55 },
            CoordPair { y: 30, x: 55 },
        )
        .unwrap();
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(graph
        .undirected_connected(
            CoordPair { y: 37, x: 55 },
            CoordPair { y: 41, x: 55 }
        )
        .unwrap());
    assert!(!graph
        .undirected_connected(
            CoordPair { y: 30, x: 55 },
            CoordPair { y: 37, x: 55 }
        )
        .unwrap());
}

#[test]
fn remove_success_connected_middle() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    graph
        .connect(CoordPair { y: 41, x: 55 }, CoordPair { y: 37, x: 55 })
        .unwrap();
    graph
        .connect(CoordPair { y: 37, x: 55 }, CoordPair { y: 30, x: 55 })
        .unwrap();
    graph.remove_node(CoordPair { y: 37, x: 55 }).unwrap();
    let connected = graph
        .connected(CoordPair { y: 41, x: 55 }, CoordPair { y: 30, x: 55 })
        .unwrap();
    assert!(connected);
}

#[test]
fn remove_success_connected_edge() {
    let mut graph = CoordDiGraph::new();
    graph.insert_node(CoordPair { y: 30, x: 55 });
    graph.insert_node(CoordPair { y: 37, x: 55 });
    graph.insert_node(CoordPair { y: 41, x: 55 });
    graph
        .connect(CoordPair { y: 41, x: 55 }, CoordPair { y: 30, x: 55 })
        .unwrap();
    graph.remove_node(CoordPair { y: 30, x: 55 }).unwrap();
    let connected = graph
        .connected(CoordPair { y: 41, x: 55 }, CoordPair { y: 37, x: 55 })
        .unwrap();
    assert!(connected);
    let node = graph.node(CoordPair { y: 41, x: 55 }.as_ref()).unwrap();
    assert!(node.connected(Direction::Up));
    let node = graph.node(CoordPair { y: 37, x: 55 }.as_ref()).unwrap();
    assert!(!node.connected(Direction::Up));
}
