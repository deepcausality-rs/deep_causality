/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Hypergraph, TopologyError, TopologyErrorEnum};

// Exercises the "at least one node and one hyperedge" rejection in
// `Hypergraph::new_impl`: an incidence matrix with zero rows (no nodes) must
// be rejected before any other validation runs.
#[test]
fn test_hypergraph_new_rejects_empty_incidence() {
    // 0 nodes, 1 hyperedge -> num_nodes == 0 triggers the InvalidInput error.
    let incidence: CsrMatrix<i8> = CsrMatrix::from_triplets(0, 1, &[]).unwrap();
    let data: CausalTensor<f64> = CausalTensor::new(vec![], vec![0]).unwrap();

    let err = Hypergraph::new(incidence, data, 0).expect_err("empty incidence must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::InvalidInput(_))
    ));
}

#[test]
fn test_hypergraph_new_rejects_no_hyperedges() {
    // 1 node, 0 hyperedges -> num_hyperedges == 0 triggers the same branch.
    let incidence: CsrMatrix<i8> = CsrMatrix::from_triplets(1, 0, &[]).unwrap();
    let data: CausalTensor<f64> = CausalTensor::new(vec![1.0], vec![1]).unwrap();

    let err = Hypergraph::new(incidence, data, 0)
        .expect_err("incidence with no hyperedges must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::InvalidInput(_))
    ));
}
