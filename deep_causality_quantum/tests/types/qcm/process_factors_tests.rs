/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

use deep_causality::utils_test::test_utils;
use deep_causality::{CausableGraph, CausaloidGraph};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{FactorSupports, ProcessFactors, QuantumErrorEnum};
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
fn test_declared_leg_dim_tells_a_registered_qubit_from_an_absent_leg() {
    let mut fs = FactorSupports::new();
    assert_eq!(fs.declared_leg_dim(5), None);
    assert_eq!(fs.leg_dim(5), 2);
    fs.declare(0, &[5]);
    assert_eq!(fs.declared_leg_dim(5), Some(2));
    fs.set_leg_dim(7, 3);
    assert_eq!(fs.declared_leg_dim(7), Some(3));
    assert_eq!(fs.declared_leg_dim(6), None);
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

    // An unfrozen graph is rejected: the frozen state is the precondition.
    assert!(FactorSupports::from_graph(&g, &pf).is_err());

    // Frozen graph → the supports read off its edges.
    g.freeze();
    let fs = FactorSupports::from_graph(&g, &pf).unwrap();
    assert_eq!(fs.support(n0), Some([0].as_slice()));
    assert_eq!(fs.support(n1), Some([1].as_slice()));
    assert_eq!(fs.support(n2), Some([0, 1, 2].as_slice()));
    assert_eq!(fs.support_dim(n2), Some(8));
    assert!(fs.validate(&pf).is_ok());
}

/// The collider `n0 → n2 ← n1` with an extra causaloid at index 2, added before `n2` and removed
/// after `n2` is added, so that `n2 = 3` is live while `number_nodes()` is 3, and the removed
/// index 2 sits below it. Returns the frozen graph and `(n0, n1, removed, n2)`.
fn frozen_collider_with_a_removed_index() -> (
    CausaloidGraph<deep_causality::BaseCausaloid<f64, bool>>,
    [usize; 4],
) {
    let mut g = CausaloidGraph::new(0);
    let n0 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(0))
        .unwrap();
    let n1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .unwrap();
    let extra = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(9))
        .unwrap();
    let n2 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .unwrap();
    g.add_edge(n0, n2).unwrap();
    g.add_edge(n1, n2).unwrap();
    g.remove_causaloid(extra).unwrap();
    g.freeze();
    (g, [n0, n1, extra, n2])
}

#[test]
fn test_from_graph_supports_a_collider_whose_sink_is_a_live_index_above_number_nodes() {
    // n2 = 3 is live and number_nodes() is 3. support(n2) = {n0, n1, n2} = {0, 1, 3}, three
    // qubits, dim 2·2·2 = 8; a check written as `node >= number_nodes()` refuses n2.
    let (g, [n0, n1, _, n2]) = frozen_collider_with_a_removed_index();
    assert_eq!(g.number_nodes(), 3);
    assert_eq!(n2, 3);

    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(n0, sigma_z());
    pf.insert(n1, sigma_z());
    pf.insert(n2, mat(vec![c(0., 0.); 64], 8));

    let fs = FactorSupports::from_graph(&g, &pf).unwrap();
    assert_eq!(fs.support(n0), Some([n0].as_slice()));
    assert_eq!(fs.support(n1), Some([n1].as_slice()));
    assert_eq!(fs.support(n2), Some([n0, n1, n2].as_slice()));
    assert_eq!(fs.support_dim(n2), Some(8));
    assert!(fs.validate(&pf).is_ok());
}

#[test]
fn test_from_graph_refuses_a_factor_keyed_by_a_removed_index_below_a_live_one() {
    // The removed index 2 is below number_nodes() == 3 yet holds no causaloid; the refusal
    // names node 2.
    let (g, [n0, _, removed, _]) = frozen_collider_with_a_removed_index();
    assert_eq!(removed, 2);

    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(n0, sigma_z());
    pf.insert(removed, sigma_z());
    match FactorSupports::from_graph(&g, &pf).unwrap_err().0 {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("node 2"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_validate_rejects_non_square_factor() {
    // A 2×3 factor is not a square operator.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(
        0,
        CausalTensor::new(vec![c(1., 0.); 6], vec![2, 3]).unwrap(),
    );
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    assert!(matches!(
        fs.validate(&pf).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_validate_rejects_factor_without_declared_support() {
    // A factor exists at node 0 but its support was never declared.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    let fs = FactorSupports::new();
    assert!(matches!(
        fs.validate(&pf).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_a_leg_has_no_output_half_until_one_is_declared() {
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    assert_eq!(fs.leg_output_dim(0), 1);
    fs.set_leg_output_dim(0, 4);
    assert_eq!(fs.leg_output_dim(0), 4);
    assert_eq!(fs.leg_output_dim(7), 1);
}

#[test]
fn test_validate_requires_an_output_half_that_divides_its_leg() {
    // A leg of dimension 4 carrying the identity factor on node 0.
    let identity4 = || {
        let mut data = vec![c(0., 0.); 16];
        for i in 0..4 {
            data[i * 4 + i] = c(1., 0.);
        }
        mat(data, 4)
    };
    let registry = |output: usize| {
        let mut fs = FactorSupports::new();
        fs.declare(0, &[0]);
        fs.set_leg_dim(0, 4).set_leg_output_dim(0, output);
        fs
    };
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, identity4());
    for output in [1, 2, 4] {
        assert!(registry(output).validate(&pf).is_ok(), "output {output}");
    }
    for output in [0, 3, 8] {
        match registry(output).validate(&pf).unwrap_err().0 {
            QuantumErrorEnum::DimensionMismatch(msg) => {
                assert!(
                    msg.contains("leg 0") && msg.contains(&format!("{output}")),
                    "{msg}"
                )
            }
            other => panic!("expected DimensionMismatch for output {output}, got {other:?}"),
        }
    }
    // A parent leg's output half is checked as well, and a leg no factor uses is not.
    let mut fs = FactorSupports::new();
    fs.declare(1, &[0, 1]);
    fs.set_leg_dim(0, 2)
        .set_leg_dim(1, 2)
        .set_leg_output_dim(0, 3)
        .set_leg_output_dim(9, 0);
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(1, identity4());
    assert!(fs.validate(&pf).is_err());
    fs.set_leg_output_dim(0, 2);
    assert!(fs.validate(&pf).is_ok());
}
