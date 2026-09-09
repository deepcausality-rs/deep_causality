/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_homology::Gf2Chain;
use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    GateOp, LogicalProgram, apply_gate, born_probability, commutator, expectation_value, fidelity,
    haruna_cz_gate, haruna_hadamard_gate, haruna_s_gate, haruna_t_gate, haruna_x_gate,
    haruna_z_gate,
};

// The wrappers are thin adapters over `mechanics`, so the risk they carry is
// substitution: lifting the wrong kernel, dropping the value, or carrying a
// default.
//
// These tests therefore assert the same independently derived values as
// mechanics_tests.rs, whose module header records how they are obtained from
// the Clifford relations. Blade indices are the bitmap order documented there:
// 0 is the scalar 1, 1 is e1, 2 is e2, 3 is e1e2.

/// A state from its eight blade coefficients.
fn state8(data: [Complex<f64>; 8]) -> HilbertState<f64> {
    HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data.to_vec(), Metric::Euclidean(3)).unwrap(),
    )
}

/// The unit blade at `index`.
fn blade(index: usize) -> HilbertState<f64> {
    let mut data = [Complex::new(0.0, 0.0); 8];
    data[index] = Complex::new(1.0, 0.0);
    state8(data)
}

/// psi = 0.6 + 0.8i e1, unit norm.
fn psi_complex() -> HilbertState<f64> {
    state8([
        Complex::new(0.6, 0.0),
        Complex::new(0.0, 0.8),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ])
}

/// phi = 0.6i + 0.8 e1, unit norm. |<psi|phi>|^2 = 0.0784.
fn phi_complex() -> HilbertState<f64> {
    state8([
        Complex::new(0.0, 0.6),
        Complex::new(0.8, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ])
}

fn real_of(effect: deep_causality_core::PropagatingEffect<f64>) -> f64 {
    effect
        .value_cloned()
        .expect("the effect should carry a value")
}

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
    // Identical unit states carry P = 1, not merely "some Ok value".
    let state = create_test_state();
    let basis = create_test_state();
    assert!((real_of(born_probability(&state, &basis)) - 1.0).abs() < 1e-12);

    // And an off-diagonal pair carries the exact overlap 0.0784, which a
    // wrapper returning a constant, a default, or the other kernel would miss.
    let p = real_of(born_probability(&psi_complex(), &phi_complex()));
    assert!((p - 0.0784).abs() < 1e-12, "expected 0.0784, got {p}");
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
    // <psi|1|psi> = 1 for a unit state.
    let state = create_test_state();
    let operator = create_test_state();
    assert!((real_of(expectation_value(&state, &operator)) - 1.0).abs() < 1e-12);

    // psi = (1 + e1 + e2 - e1e2)/2 is a unit eigenvector of e2 with eigenvalue
    // +1, so the wrapper must carry 1 and not the 0 the right action gives.
    let h = 0.5;
    let eigen = state8([
        Complex::new(h, 0.0),
        Complex::new(h, 0.0),
        Complex::new(h, 0.0),
        Complex::new(-h, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ]);
    let v = real_of(expectation_value(&eigen, &blade(2)));
    assert!((v - 1.0).abs() < 1e-12, "expected 1, got {v}");
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
    // U = e2 on psi = 0.6 + 0.8 e1 carries 0.6 e2 - 0.8 e1e2. The sign on blade
    // 3 is what separates the left action from the right one; `is_ok()` saw
    // neither, and the old fixture was the identity, where they agree.
    let psi = state8([
        Complex::new(0.6, 0.0),
        Complex::new(0.8, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ]);
    let out = apply_gate(&psi, &blade(2))
        .value_cloned()
        .expect("the effect should carry a state");
    let got = out.mv().data();
    assert!((got[2].re - 0.6).abs() < 1e-12, "blade 2: {:?}", got[2]);
    assert!((got[3].re + 0.8).abs() < 1e-12, "blade 3: {:?}", got[3]);
    assert!(got[0].re.abs() < 1e-12 && got[1].re.abs() < 1e-12);
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
    // [e1, e2] = 2 e1e2, so blade 3 carries 2. A wrapper carrying the zero
    // state — which the old identity fixture would have accepted, since
    // [1, 1] = 0 — fails here.
    let out = commutator(&blade(1), &blade(2))
        .value_cloned()
        .expect("the effect should carry a state");
    let got = out.mv().data();
    assert!((got[3].re - 2.0).abs() < 1e-12, "blade 3: {:?}", got[3]);
    for (i, c) in got.iter().enumerate().filter(|(i, _)| *i != 3) {
        assert!(c.re.abs() < 1e-12 && c.im.abs() < 1e-12, "blade {i}: {c:?}");
    }
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
    // Identical states carry F = 1; a distinct pair carries the exact overlap.
    // Without the second case a wrapper hard-wired to 1 passes.
    let ideal = create_test_state();
    let actual = create_test_state();
    assert!((real_of(fidelity(&ideal, &actual)) - 1.0).abs() < 1e-12);

    let f = real_of(fidelity(&psi_complex(), &phi_complex()));
    assert!((f - 0.0784).abs() < 1e-12, "expected 0.0784, got {f}");
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
fn test_haruna_hadamard_wrapper_carries_the_program_and_its_phase() {
    let g = chain(&[0, 1]);
    let gt = chain(&[2]);
    // S(g) is 3 ops, H over supp(gt) is 1, S(gt) is 1, H again 1, S(g) again 3.
    let program = haruna_hadamard_gate::<u64, f64>(&g, &gt)
        .value_cloned()
        .expect("the effect should carry a program");
    assert_eq!(program.len(), 9);
    // Table 1's e^{-iπ/4}, kept rather than dropped.
    let phase = program
        .global_phase()
        .expect("the Hadamard carries a phase");
    let expected = std::f64::consts::FRAC_PI_4;
    assert!((phase.re - expected.cos()).abs() < 1e-12);
    assert!((phase.im + expected.sin()).abs() < 1e-12);
    // And the diagonal programs carry none.
    let diagonal = LogicalProgram::<f64>::from(value_of(haruna_s_gate(&g)));
    assert!(diagonal.global_phase().is_none());
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
