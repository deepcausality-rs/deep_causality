/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::{BrcdErrorEnum, dag_to_cpdag};
use deep_causality_topology::EdgeKind;

#[test]
fn chain_has_no_v_structure_so_all_edges_are_undirected() {
    // X(0) -> Y(1) -> Z(2): no node has two non-adjacent parents.
    let cpdag = dag_to_cpdag(&[vec![], vec![0], vec![1]]).unwrap();
    assert_eq!(cpdag.edge_kind(0, 1), Some(EdgeKind::Undirected));
    assert_eq!(cpdag.edge_kind(1, 2), Some(EdgeKind::Undirected));
    assert_eq!(cpdag.edge_kind(0, 2), None);
}

#[test]
fn fork_is_fully_undirected() {
    // Z(2) -> X(0), Z(2) -> Y(1): X←Z→Y, again no collider.
    let cpdag = dag_to_cpdag(&[vec![2], vec![2], vec![]]).unwrap();
    assert_eq!(cpdag.edge_kind(0, 2), Some(EdgeKind::Undirected));
    assert_eq!(cpdag.edge_kind(1, 2), Some(EdgeKind::Undirected));
    assert_eq!(cpdag.edge_kind(0, 1), None);
}

#[test]
fn unshielded_collider_stays_directed() {
    // X(0) -> Z(2) <- Y(1), X and Y non-adjacent: the v-structure is compelled.
    let cpdag = dag_to_cpdag(&[vec![], vec![], vec![0, 1]]).unwrap();
    assert_eq!(cpdag.edge_kind(0, 2), Some(EdgeKind::Directed));
    assert_eq!(cpdag.edge_kind(1, 2), Some(EdgeKind::Directed));
    // Direction is into the collider.
    let parents_of_z = cpdag.parents(2);
    assert!(parents_of_z.contains(&0) && parents_of_z.contains(&1));
    assert_eq!(cpdag.edge_kind(0, 1), None);
}

#[test]
fn shielded_triple_has_no_v_structure() {
    // Complete DAG X->Y, X->Z, Y->Z: Z's parents X,Y are adjacent (shielded),
    // so there is no collider and the essential graph is the undirected triangle.
    let cpdag = dag_to_cpdag(&[vec![], vec![0], vec![0, 1]]).unwrap();
    assert_eq!(cpdag.edge_kind(0, 1), Some(EdgeKind::Undirected));
    assert_eq!(cpdag.edge_kind(0, 2), Some(EdgeKind::Undirected));
    assert_eq!(cpdag.edge_kind(1, 2), Some(EdgeKind::Undirected));
}

#[test]
fn meek_propagates_beyond_the_v_structure() {
    // X(0)->Z(2)<-Y(1) collider, plus Z(2)->W(3). After orienting the collider,
    // Meek R1 forces Z->W (orienting W->Z would make a new collider at Z).
    let cpdag = dag_to_cpdag(&[vec![], vec![], vec![0, 1], vec![2]]).unwrap();
    assert_eq!(cpdag.edge_kind(0, 2), Some(EdgeKind::Directed));
    assert_eq!(cpdag.edge_kind(1, 2), Some(EdgeKind::Directed));
    assert_eq!(cpdag.edge_kind(2, 3), Some(EdgeKind::Directed));
    assert!(cpdag.parents(3).contains(&2), "Z -> W must be compelled");
}

#[test]
fn empty_parent_list_is_empty_data() {
    let err = dag_to_cpdag(&[]).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::EmptyData);
}

#[test]
fn parent_index_out_of_bounds_is_an_error() {
    // Node 0 names a parent 5 that does not exist.
    let err = dag_to_cpdag(&[vec![5], vec![]]).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::NodeOutOfBounds);
}

#[test]
fn self_loop_is_not_acyclic() {
    let err = dag_to_cpdag(&[vec![0]]).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::NotAcyclic);
}

#[test]
fn directed_cycle_is_not_acyclic() {
    // 0 -> 1 and 1 -> 0.
    let err = dag_to_cpdag(&[vec![1], vec![0]]).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::NotAcyclic);
}

#[test]
fn single_node_yields_an_edgeless_cpdag() {
    let cpdag = dag_to_cpdag(&[vec![]]).unwrap();
    assert_eq!(cpdag.num_vertices(), 1);
    assert_eq!(cpdag.num_edges(), 0);
}
