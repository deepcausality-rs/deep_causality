/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_linear::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{GraphTopology, Hypergraph};

fn create_simple_hypergraph() -> Hypergraph<f64> {
    // Hyperedge 0: {0, 1}
    // Hyperedge 1: {1, 2}
    let incidence =
        CsrMatrix::from_triplets(3, 2, &[(0, 0, 1i8), (1, 0, 1), (1, 1, 1), (2, 1, 1)]).unwrap();

    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    Hypergraph::new(incidence, data, 0).unwrap()
}

#[test]
fn test_hypergraph_graph_topology() {
    let hg = create_simple_hypergraph();

    // Num nodes/edges
    assert_eq!(hg.num_nodes(), 3);
    assert_eq!(hg.num_edges(), 2); // 2 hyperedges

    // has_node
    assert!(hg.has_node(0));
    assert!(hg.has_node(2));
    assert!(!hg.has_node(3));

    // get_neighbors
    // Node 0 is in edge 0 {0,1}, neighbor is 1
    let neighbors0 = hg.get_neighbors(0).unwrap();
    assert_eq!(neighbors0.len(), 1);
    assert_eq!(neighbors0[0], 1);

    // Node 1 is in edge 0 {0,1} and edge 1 {1,2}, neighbors 0 and 2
    let neighbors1 = hg.get_neighbors(1).unwrap();
    assert_eq!(neighbors1.len(), 2);
    assert!(neighbors1.contains(&0));
    assert!(neighbors1.contains(&2));

    // Node 2 is in edge 1 {1,2}, neighbor is 1
    let neighbors2 = hg.get_neighbors(2).unwrap();
    assert_eq!(neighbors2.len(), 1);
    assert_eq!(neighbors2[0], 1);

    // Error check
    assert!(hg.get_neighbors(99).is_err());
}

#[test]
fn test_get_neighbors_skips_a_stored_zero_incidence_entry() {
    // An incidence matrix is 0/1 valued: a stored 0 means the node is not in
    // that hyperedge. `from_triplets_with_zero` keeps such an entry in the
    // sparse pattern, so the traversal has to read the value rather than treat
    // every stored position as a membership.
    //
    // Hyperedge 0: {0, 1}, with an explicit non-membership recorded for node 2.
    // Hyperedge 1: {2} alone.
    let incidence = CsrMatrix::from_triplets_with_zero(
        3,
        2,
        &[(0, 0, 1i8), (1, 0, 1), (2, 0, 0), (2, 1, 1)],
        7i8,
    )
    .unwrap();
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let hg = Hypergraph::new(incidence, data, 0).unwrap();

    assert_eq!(hg.get_neighbors(0).unwrap(), vec![1]);
    assert_eq!(hg.get_neighbors(1).unwrap(), vec![0]);
    // Node 2 is recorded against hyperedge 0 with value 0, so it is not a
    // member and gains no neighbour from it; hyperedge 1 holds it alone.
    assert!(hg.get_neighbors(2).unwrap().is_empty());
}
