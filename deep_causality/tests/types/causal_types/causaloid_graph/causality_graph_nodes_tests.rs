/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use ultragraph::*;

use deep_causality::utils_test::test_utils;

// Custom type alias

fn get_causal_graph() -> BaseCausalGraph {
    let g: BaseCausalGraph = CausaloidGraph::new(0);
    g
}

#[test]
fn test_add_root_causaloid() {
    let mut g = get_causal_graph();
    let root_causaloid = test_utils::get_test_causaloid();

    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);
}

#[test]
fn test_add_root_causaloid_err() {
    let mut g = get_causal_graph();
    let root_causaloid = test_utils::get_test_causaloid();

    let root_index = g
        .add_root_causaloid(root_causaloid.clone())
        .expect("Failed to add root index");
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    let res = g.add_root_causaloid(root_causaloid.clone());
    assert!(res.is_err());
}

#[test]
fn test_get_root_causaloid() {
    let mut g = get_causal_graph();
    let root_causaloid = test_utils::get_test_causaloid();

    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    let causaloid = g.get_root_causaloid().unwrap();

    let id = causaloid.id();
    assert_eq!(id, 1);
}

#[test]
fn test_get_root_index() {
    let mut g = get_causal_graph();
    let root_causaloid = test_utils::get_test_causaloid();

    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    let r_index = g.get_root_index().unwrap();
    assert_eq!(root_index, r_index);
}

#[test]
fn test_get_last_index() {
    let mut g = get_causal_graph();

    let res = g.get_last_index();
    assert!(res.is_err());

    let root_causaloid = test_utils::get_test_causaloid();

    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    let r_index = g.get_root_index().unwrap();
    assert_eq!(root_index, r_index);

    let res = g.get_last_index();
    assert!(res.is_ok());

    let res = g.remove_causaloid(root_index);
    assert!(res.is_ok());

    let res = g.get_last_index();
    assert!(res.is_err());
}

#[test]
fn test_add_causaloid() {
    let mut g = get_causal_graph();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains = g.contains_causaloid(index);
    assert!(contains);
}

#[test]
fn test_contains_causaloid() {
    let mut g = get_causal_graph();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains = g.contains_causaloid(index);
    assert!(contains);
}

#[test]
fn test_get_causaloid() {
    let mut g = get_causal_graph();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains = g.contains_causaloid(index);
    assert!(contains);

    let causaloid = g.get_causaloid(index).unwrap();

    let id = causaloid.id();
    let description = causaloid.description();

    assert_eq!(id, 1);
    assert_eq!(description, "tests whether data exceeds threshold of 0.55");
}

#[test]
fn test_remove_causaloid() {
    let mut g = get_causal_graph();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains = g.contains_causaloid(index);
    assert!(contains);

    let causaloid = g.get_causaloid(index).unwrap();

    let id = causaloid.id();
    let description = causaloid.description();

    assert_eq!(id, 1);
    assert_eq!(description, "tests whether data exceeds threshold of 0.55");

    let res = g.remove_causaloid(index);
    assert!(res.is_ok());

    let contains = g.contains_causaloid(index);
    assert!(!contains);
}

#[test]
fn test_get_graph() {
    let g = get_causal_graph();

    let size = g.size();
    assert_eq!(size, 0);

    let graph = g.get_graph();
    assert_eq!(graph.number_edges(), 0);
    assert_eq!(graph.number_nodes(), 0);
}
