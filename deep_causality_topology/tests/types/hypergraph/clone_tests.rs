/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Hypergraph;

fn create_simple_hypergraph() -> Hypergraph<f64> {
    let incidence = CsrMatrix::from_triplets(3, 2, &[(0, 0, 1i8), (1, 0, 1)]).unwrap();

    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    Hypergraph::new(incidence, data, 0).unwrap()
}

#[test]
fn test_hypergraph_clone_shallow() {
    let hg = create_simple_hypergraph();
    let cloned = hg.clone_shallow();

    assert_eq!(hg.num_nodes(), cloned.num_nodes());
    assert_eq!(hg.num_hyperedges(), cloned.num_hyperedges());

    // Ensure independency (though shallow clone usually shares data reference if using Arc,
    // here the struct owns fields, but Tensor/Matrix might share backend or handle clone efficiently)
    // The main point is that it produced a valid object.
}
