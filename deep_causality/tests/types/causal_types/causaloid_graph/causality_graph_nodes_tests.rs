/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use ultragraph::*;

use deep_causality::utils_test::test_utils;

#[test]
fn test_add_root_causaloid() {
    let mut g = CausaloidGraph::new(0);
    let root_causaloid = test_utils::get_test_causaloid_deterministic(0);

    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);
}

#[test]
fn test_add_root_causaloid_err() {
    let mut g = CausaloidGraph::new(0);
    let root_causaloid = test_utils::get_test_causaloid_deterministic(0);

    let root_index = g
        .add_root_causaloid(root_causaloid.clone())
        .expect("Failed to add root index");
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    let res = g.add_root_causaloid(root_causaloid);
    assert!(res.is_err());
}

#[test]
fn test_get_root_causaloid() {
    let mut g = CausaloidGraph::new(0);
    let root_causaloid = test_utils::get_test_causaloid_deterministic(0);

    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    let id = g.get_root_causaloid().unwrap().id();

    assert_eq!(id, 0);
}

#[test]
fn test_get_root_index() {
    let mut g = CausaloidGraph::new(0);
    let root_causaloid = test_utils::get_test_causaloid_deterministic(0);

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
    let mut g = CausaloidGraph::new(0);

    let res = g.get_last_index();
    assert!(res.is_err());

    let root_causaloid = test_utils::get_test_causaloid_deterministic(0);

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
    let mut g = CausaloidGraph::new(0);
    let causaloid = test_utils::get_test_causaloid_deterministic(1);

    let index = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains = g.contains_causaloid(index);
    assert!(contains);
}

#[test]
fn test_contains_causaloid() {
    let mut g = CausaloidGraph::new(0);
    let causaloid = test_utils::get_test_causaloid_deterministic(1);

    let index = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains = g.contains_causaloid(index);
    assert!(contains);
}

#[test]
fn test_get_causaloid() {
    let mut g = CausaloidGraph::new(0);
    let causaloid = test_utils::get_test_causaloid_deterministic(1);

    let index = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains = g.contains_causaloid(index);
    assert!(contains);

    let id = g.get_causaloid(index).unwrap().id();

    assert_eq!(id, 1);
}

#[test]
fn test_remove_causaloid() {
    let mut g = CausaloidGraph::new(0);
    let causaloid = test_utils::get_test_causaloid_deterministic(1);

    let index = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains = g.contains_causaloid(index);
    assert!(contains);

    let id = g.get_causaloid(index).unwrap().id();
    assert_eq!(id, 1);

    let res = g.remove_causaloid(index);
    assert!(res.is_ok());

    let contains = g.contains_causaloid(index);
    assert!(!contains);
}

#[test]
fn test_get_graph() {
    let g: CausaloidGraph<BaseCausaloid<NumericalValue, bool>> = CausaloidGraph::new(0);

    let size = g.size();
    assert_eq!(size, 0);

    let graph = g.get_graph();
    assert_eq!(graph.number_edges(), 0);
    assert_eq!(graph.number_nodes(), 0);
}

#[test]
fn test_contains_root_causaloid_empty() {
    let g = CausaloidGraph::<BaseCausaloid<NumericalValue, bool>>::new(0);
    assert!(!g.contains_root_causaloid());
}

#[test]
fn test_get_root_causaloid_empty() {
    let g = CausaloidGraph::<BaseCausaloid<NumericalValue, bool>>::new(0);
    assert!(g.get_root_causaloid().is_none());
}

#[test]
fn test_is_empty_and_clear() {
    let mut g = CausaloidGraph::new(0);
    assert!(g.is_empty());

    let causaloid = test_utils::get_test_causaloid_deterministic(1);
    g.add_causaloid(causaloid).unwrap();
    assert!(!g.is_empty());
    assert_eq!(g.size(), 1);
    assert_eq!(g.number_nodes(), 1);

    g.clear();
    assert!(g.is_empty());
    assert_eq!(g.size(), 0);
}

#[test]
fn test_remove_causaloid_error() {
    let mut g = CausaloidGraph::<BaseCausaloid<NumericalValue, bool>>::new(0);
    let res = g.remove_causaloid(99); // Invalid index
    assert!(res.is_err());
}
