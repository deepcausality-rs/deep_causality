/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, Hypergraph};

fn create_simple_hypergraph() -> Hypergraph<f64> {
    let incidence = CsrMatrix::from_triplets(
        3,
        2,
        &[
            (0, 0, 1i8), // node 0 in hyperedge 0
            (1, 0, 1),   // node 1 in hyperedge 0
            (1, 1, 1),   // node 1 in hyperedge 1
            (2, 1, 1),   // node 2 in hyperedge 1
        ],
    )
    .unwrap();

    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    Hypergraph::new(incidence, data, 0).unwrap()
}

#[test]
fn test_hypergraph_base_topology() {
    let hg = create_simple_hypergraph();

    // Dimension should be 1
    assert_eq!(hg.dimension(), 1);

    // Len should be num_nodes = 3
    assert_eq!(hg.len(), 3);

    // Num elements at grade
    assert_eq!(hg.num_elements_at_grade(0), Some(3)); // nodes
    assert_eq!(hg.num_elements_at_grade(1), Some(2)); // hyperedges
    assert_eq!(hg.num_elements_at_grade(2), None);
}
