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
