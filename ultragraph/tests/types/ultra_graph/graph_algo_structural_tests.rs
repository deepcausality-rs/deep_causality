/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use ultragraph::utils::utils_tests;
use ultragraph::{GraphMut, StructuralGraphAlgorithms, UltraGraph, UltraGraphWeighted};

#[test]
fn test_strongly_connected_components() {
    let mut g = UltraGraphWeighted::new();
    // SCC 1: {0, 1, 2}
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 0, ()).unwrap();

    // SCC 2: {3, 4}
    g.add_node(3).unwrap();
    g.add_node(4).unwrap();
    g.add_edge(3, 4, ()).unwrap();
    g.add_edge(4, 3, ()).unwrap();

    // SCC 3: {5, 6}
    g.add_node(5).unwrap();
    g.add_node(6).unwrap();
    g.add_edge(5, 6, ()).unwrap();
    g.add_edge(6, 5, ()).unwrap();

    // SCC 4: {7} (self-loop)
    g.add_node(7).unwrap();
    g.add_edge(7, 7, ()).unwrap();

    // SCC 5: {8} (isolated node)
    g.add_node(8).unwrap();

    // Edges connecting SCCs (should not merge them)
    g.add_edge(2, 3, ()).unwrap();
    g.add_edge(4, 5, ()).unwrap();
    g.add_edge(6, 7, ()).unwrap();

    // triggers GraphNotFrozen error.
    let res = g.strongly_connected_components();
    assert!(res.is_err());

    // Freeze the graph to enable all algorithms.
    g.freeze();

    let mut sccs = g.strongly_connected_components().unwrap();

    // Sort each SCC internally and then sort the list of SCCs for consistent comparison
    for scc in sccs.iter_mut() {
        scc.sort_unstable();
    }
    sccs.sort_unstable_by_key(|scc| scc[0]); // Sort by the first element of each SCC

    let mut expected_sccs = vec![vec![0, 1, 2], vec![3, 4], vec![5, 6], vec![7], vec![8]];
    for scc in expected_sccs.iter_mut() {
        scc.sort_unstable();
    }
    expected_sccs.sort_unstable_by_key(|scc| scc[0]);

    assert_eq!(sccs, expected_sccs);

    // Test with an empty graph
    let mut g_empty = UltraGraphWeighted::<i32, i32>::new();
    g_empty.freeze();
    let sccs_empty = g_empty.strongly_connected_components().unwrap();
    assert!(sccs_empty.is_empty());

    // Test with a single node graph
    let mut g_single = UltraGraph::new();
    g_single.add_node(0).unwrap();
    g_single.freeze();
    let sccs_single = g_single.strongly_connected_components().unwrap();
    assert_eq!(sccs_single, vec![vec![0]]);

    // Test with a DAG
    let g_dag = utils_tests::get_acyclic_graph();
    let mut sccs_dag = g_dag.strongly_connected_components().unwrap();
    for scc in sccs_dag.iter_mut() {
        scc.sort_unstable();
    }
    sccs_dag.sort_unstable_by_key(|scc| scc[0]);
    let mut expected_sccs_dag = vec![vec![0], vec![1], vec![2], vec![3]];
    for scc in expected_sccs_dag.iter_mut() {
        scc.sort_unstable();
    }
    expected_sccs_dag.sort_unstable_by_key(|scc| scc[0]);
    assert_eq!(sccs_dag, expected_sccs_dag);
}
