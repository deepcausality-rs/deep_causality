/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::mec::{MecError, mec_size, representative_dag};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

#[test]
fn fully_directed_dag_has_class_size_one() {
    // 0 → 1 → 2, 0 → 2 : already a DAG, equivalence class size 1.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(0, 2).unwrap();
    assert_eq!(mec_size(&g), Ok(1));
}

#[test]
fn empty_graph_has_class_size_one() {
    // No edges: trivially fully directed and acyclic.
    let g = graph(3);
    assert_eq!(mec_size(&g), Ok(1));
}

#[test]
fn representative_of_dag_is_the_input() {
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    let rep = representative_dag(&g).unwrap();
    assert_eq!(rep.edges(), g.edges());
    assert_eq!(rep.num_vertices(), g.num_vertices());
}

#[test]
fn undirected_edge_requires_the_uniform_sampler() {
    // An undirected edge means the class has > 1 member.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    assert_eq!(mec_size(&g), Err(MecError::RequiresUniformSampler));
    assert_eq!(
        representative_dag(&g).err(),
        Some(MecError::RequiresUniformSampler)
    );
}

#[test]
fn bidirected_edge_requires_the_uniform_sampler() {
    let mut g = graph(2);
    g.add_bidirected(0, 1).unwrap();
    assert_eq!(mec_size(&g), Err(MecError::RequiresUniformSampler));
}

#[test]
fn cyclic_arc_projection_is_not_a_dag() {
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(2, 0).unwrap();
    assert_eq!(mec_size(&g), Err(MecError::NotAcyclic));
}

#[test]
fn representative_of_cyclic_graph_errors_not_acyclic() {
    // representative_dag mirrors mec_size's errors: a cyclic arc projection
    // (fully directed but not a DAG) yields NotAcyclic, not a clone.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(2, 0).unwrap();
    assert_eq!(representative_dag(&g).err(), Some(MecError::NotAcyclic));
}
