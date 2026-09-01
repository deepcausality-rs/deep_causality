/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_homology::Gf2Chain;
use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    GateOp, apply_gate, born_probability, commutator, expectation_value, fidelity, haruna_cz_gate,
    haruna_hadamard_gate, haruna_s_gate, haruna_t_gate, haruna_x_gate, haruna_z_gate,
};

fn create_test_state() -> HilbertState<f64> {
    let data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    let mv = CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap();
    HilbertState::<f64>::from_multivector(mv)
}

#[test]
fn test_born_probability_wrapper_success() {
    let state = create_test_state();
    let basis = create_test_state();

    let effect = born_probability(&state, &basis);
    assert!(effect.is_ok());
}

#[test]
fn test_born_probability_wrapper_error() {
    // An unnormalized state with large magnitude drives |<psi|psi>|^2 far
    // above 1, tripping the kernel's NormalizationError guard.
    let data = vec![Complex::new(100.0, 0.0); 8];
    let state = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap(),
    );
    let basis = state.clone();

    let effect = born_probability(&state, &basis);
    assert!(effect.is_err());
}

#[test]
fn test_expectation_value_wrapper_success() {
    let state = create_test_state();
    let operator = create_test_state();

    let effect = expectation_value(&state, &operator);
    assert!(effect.is_ok());
}

#[test]
fn test_expectation_value_wrapper_error() {
    let state = create_test_state();
    let data = vec![Complex::new(0.0, 0.0); 4]; // 2D metric
    let operator = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap(),
    );

    // Metric mismatch should trigger error
    let effect = expectation_value(&state, &operator);
    assert!(effect.is_err());
}

#[test]
fn test_apply_gate_wrapper_success() {
    let state = create_test_state();
    let gate = create_test_state();

    let effect = apply_gate(&state, &gate);
    assert!(effect.is_ok());
}

#[test]
fn test_apply_gate_wrapper_error() {
    let state = create_test_state();
    let data = vec![Complex::new(0.0, 0.0); 4]; // 2D metric
    let gate = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap(),
    );

    // Metric mismatch
    let effect = apply_gate(&state, &gate);
    assert!(effect.is_err());
}

#[test]
fn test_commutator_wrapper_success() {
    let a = create_test_state();
    let b = create_test_state();

    let effect = commutator(&a, &b);
    assert!(effect.is_ok());
}

#[test]
fn test_commutator_wrapper_error() {
    let a = create_test_state();
    let data = vec![Complex::new(0.0, 0.0); 4]; // 2D metric
    let b = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap(),
    );

    // Metric mismatch
    let effect = commutator(&a, &b);
    assert!(effect.is_err());
}

#[test]
fn test_fidelity_wrapper_success() {
    let ideal = create_test_state();
    let actual = create_test_state();

    let effect = fidelity(&ideal, &actual);
    assert!(effect.is_ok());
}

#[test]
fn test_fidelity_wrapper_error() {
    // Fidelity uses born_probability internally.
    // Trigger normalization error with large magnitude state.
    let data = vec![Complex::new(100.0, 0.0); 8];
    let ideal = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap(),
    );
    let actual = ideal.clone();

    let effect = fidelity(&ideal, &actual);
    assert!(effect.is_err());
}

// ---------------------------------------------------------------------------
// Haruna logical gates on the causal monad.
//
// The wrappers carry the physical-gate program produced by Table 1's second
// column. These tests check the monadic plumbing: that a well-formed chain
// reaches `pure` with the same program the builder returns, and that a
// mismatched pair reaches `from_error`. The gate contents themselves are pinned
// against the paper in gates_haruna_tests.rs.
// ---------------------------------------------------------------------------

type C = Gf2Chain<u64>;

/// A 6-qubit register with the given qubits in the support.
fn chain(support: &[usize]) -> C {
    C::from_support(6, 1, support).unwrap()
}

fn value_of(effect: deep_causality_core::PropagatingEffect<Vec<GateOp>>) -> Vec<GateOp> {
    effect
        .value_cloned()
        .expect("the effect should carry a gate program")
}

#[test]
fn test_haruna_single_chain_wrappers_carry_the_program() {
    let g = chain(&[0, 2]);
    // Z and X are transversal, so the program is one gate per support element.
    assert_eq!(value_of(haruna_z_gate(&g)).len(), 2);
    assert_eq!(value_of(haruna_x_gate(&g)).len(), 2);
    // S is 2 transversal + C(2,2) = 1 pair.
    assert_eq!(value_of(haruna_s_gate(&g)).len(), 3);
    // T is 2 transversal + 1 pair + 0 triples.
    assert_eq!(value_of(haruna_t_gate(&g)).len(), 3);
}

#[test]
fn test_haruna_cz_wrapper_succeeds_on_matching_registers() {
    let a = chain(&[0]);
    let b = chain(&[3]);
    assert_eq!(value_of(haruna_cz_gate(&a, &b)).len(), 1);
}

#[test]
fn test_haruna_cz_wrapper_errors_on_mismatched_registers() {
    let a = chain(&[0]);
    let wrong = C::from_support(8, 1, &[3]).unwrap();
    assert!(haruna_cz_gate(&a, &wrong).value().is_none());
}

#[test]
fn test_haruna_hadamard_wrapper_succeeds_and_drops_the_phase() {
    let g = chain(&[0, 1]);
    let gt = chain(&[2]);
    // S(g) is 3 ops, H over supp(gt) is 1, S(gt) is 1, H again 1, S(g) again 3.
    let ops = value_of(haruna_hadamard_gate::<u64, f64>(&g, &gt));
    assert_eq!(ops.len(), 9);
}

#[test]
fn test_haruna_hadamard_wrapper_errors_on_mismatched_registers() {
    let g = chain(&[0]);
    let wrong = C::from_support(8, 1, &[2]).unwrap();
    assert!(
        haruna_hadamard_gate::<u64, f64>(&g, &wrong)
            .value()
            .is_none()
    );
}
