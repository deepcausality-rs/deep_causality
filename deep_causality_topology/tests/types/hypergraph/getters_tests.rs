/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Hypergraph;

#[test]
fn test_hypergraph_getters() {
    let incidence = CsrMatrix::from_triplets(3, 2, &[(0, 0, 1i8), (1, 0, 1)]).unwrap();

    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let hg = Hypergraph::new(incidence, data, 0).unwrap();

    assert_eq!(hg.num_nodes(), 3);
    assert_eq!(hg.num_hyperedges(), 2);

    // Incidence matrix getter
    assert_eq!(hg.incidence().shape(), (3, 2));

    // Data tensor getter
    assert_eq!(hg.data().shape(), &[3]);
}
