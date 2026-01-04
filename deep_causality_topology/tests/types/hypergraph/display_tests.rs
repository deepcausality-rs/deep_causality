/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Hypergraph;

#[test]
fn test_hypergraph_display() {
    let incidence = CsrMatrix::from_triplets(3, 2, &[(0, 0, 1i8)]).unwrap();

    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let hg = Hypergraph::new(incidence, data, 0).unwrap();

    let display_str = format!("{}", hg);
    assert!(display_str.contains("Hypergraph { nodes: 3, hyperedges: 2 }"));
}
