/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::{CausableGraph, CausaloidGraph};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{FactorSupports, ProcessFactors};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn mat(data: Vec<C>, d: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![d, d]).unwrap()
}

fn sigma_z() -> CausalTensor<C> {
    mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(-1., 0.)], 2)
}

#[test]
fn test_process_factors_insert_get_nodes() {
    let mut pf = ProcessFactors::<f64>::new();
    assert!(pf.is_empty());
    pf.insert(2, sigma_z());
    pf.insert(0, sigma_z());
    assert_eq!(pf.len(), 2);
    assert!(!pf.is_empty());
    assert!(pf.get(0).is_some());
    assert!(pf.get(1).is_none());
    // Keys come back ascending, not insertion order.
    let nodes: Vec<usize> = pf.nodes().collect();
    assert_eq!(nodes, vec![0, 2]);
}

#[test]
fn test_factor_supports_declare_and_dims() {
    let mut fs = FactorSupports::new();
    fs.declare(5, &[3, 1]); // unsorted input → stored ascending
    assert_eq!(fs.support(5), Some([1, 3].as_slice()));
    assert_eq!(fs.leg_dim(1), 2); // qubit default
    assert_eq!(fs.support_dim(5), Some(4)); // 2 * 2
    fs.set_leg_dim(3, 3);
    assert_eq!(fs.support_dim(5), Some(6)); // 2 * 3
}

#[test]
fn test_validate_dimension_agreement() {
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z()); // 2x2
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]); // single qubit leg → dim 2
    assert!(fs.validate(&pf).is_ok());

    // A factor whose dim disagrees with its declared support is rejected.
    let mut fs_bad = FactorSupports::new();
    fs_bad.declare(0, &[0, 1]); // two qubits → dim 4, but factor is 2x2
    assert!(fs_bad.validate(&pf).is_err());
}

#[test]
fn test_from_graph_builds_collider_supports() {
    // Collider: 0 → 2, 1 → 2. support(2) = {0,1,2}, support(0)={0}, support(1)={1}.
    let mut g = CausaloidGraph::new(0);
    let n0 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(0))
        .unwrap();
    let n1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .unwrap();
    let n2 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .unwrap();
    g.add_edge(n0, n2).unwrap();
    g.add_edge(n1, n2).unwrap();

    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(n0, sigma_z());
    pf.insert(n1, sigma_z());
    // node 2's factor is on 3 qubits → 8x8.
    pf.insert(n2, mat(vec![c(0., 0.); 64], 8));

    // A dynamic graph is rejected (its node-id space may be sparse).
    assert!(FactorSupports::from_graph(&g, &pf).is_err());

    // Frozen graph → dense ids → correct supports.
    g.freeze();
    let fs = FactorSupports::from_graph(&g, &pf).unwrap();
    assert_eq!(fs.support(n0), Some([0].as_slice()));
    assert_eq!(fs.support(n1), Some([1].as_slice()));
    assert_eq!(fs.support(n2), Some([0, 1, 2].as_slice()));
    assert_eq!(fs.support_dim(n2), Some(8));
    assert!(fs.validate(&pf).is_ok());
}
