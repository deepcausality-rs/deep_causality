/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    QuantumError, QuantumErrorEnum, apply_gate_kernel, born_probability_kernel, commutator_kernel,
    expectation_value_kernel, fidelity_kernel,
};

// ---------------------------------------------------------------------------
// Oracle provenance
//
// The states below live in Cl(3,0) over C, which `CausalMultiVector` stores as
// eight complex coefficients indexed by a blade BITMAP: bit k set means the
// basis vector e_{k+1} is a factor. So index 0 is the scalar 1, 1 is e1, 2 is
// e2, 3 is e1e2, 4 is e3, 5 is e1e3, 6 is e2e3, 7 is e1e2e3.
//
// Every expected value here is derived from the Clifford relations themselves —
// e_i e_j = -e_j e_i for i != j, and e_i^2 = +1 in a Euclidean signature — not
// from the kernels and not from any helper they call.
//
// Two derivations are reused:
//
//   1. For a Euclidean metric the Dirac adjoint is `dag` (reversion followed by
//      complex conjugation). Reversing a grade-k blade contributes
//      (-1)^(k(k-1)/2), and squaring that same blade in the product contributes
//      the identical factor, so the two cancel. The bracket therefore collapses
//      to the ordinary Hermitian sum
//
//          <phi|psi> = sum_i conj(phi_i) * psi_i
//
//      over the eight coefficients. Dropping the conjugate changes the MODULUS
//      whenever two components are complex, which is what the Born test below
//      is built to expose.
//
//   2. e1 * (a + b e1) = b + a e1, because e1^2 = 1. Hence
//      <psi|e1|psi> = conj(a) b + conj(b) a = 2 Re(conj(a) b), which is the
//      Pauli-X expectation value on the real superposition used below.
//
// The previous fixture was the single multivector [1,0,0,0,0,0,0,0] — the
// scalar 1, which is the multiplicative IDENTITY of the algebra — passed as
// BOTH arguments of every valid-path test. Under it every product, adjoint,
// operand order and index permutation agrees, so none of them was observed.
// ---------------------------------------------------------------------------

/// A state from its eight blade coefficients, in the bitmap order above.
fn state8(data: [Complex<f64>; 8]) -> HilbertState<f64> {
    HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data.to_vec(), Metric::Euclidean(3)).unwrap(),
    )
}

/// The unit blade at `index`, e.g. `blade(1)` is e1 and `blade(3)` is e1e2.
fn blade(index: usize) -> HilbertState<f64> {
    let mut data = [Complex::new(0.0, 0.0); 8];
    data[index] = Complex::new(1.0, 0.0);
    state8(data)
}

/// The eight coefficients of a state, for component-wise assertions.
fn coeffs(state: &HilbertState<f64>) -> Vec<Complex<f64>> {
    state.mv().data().to_vec()
}

fn assert_coeffs_close(actual: &HilbertState<f64>, expected: [Complex<f64>; 8]) {
    for (i, (a, e)) in coeffs(actual).iter().zip(expected.iter()).enumerate() {
        assert!(
            (a.re - e.re).abs() < 1e-12 && (a.im - e.im).abs() < 1e-12,
            "blade {i}: expected {e:?}, got {a:?}"
        );
    }
}

// Helper to create a normalized quantum state
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

// =============================================================================
// Born Probability Kernel Tests
// =============================================================================

#[test]
fn test_born_probability_kernel_normalized() {
    let state = create_test_state();
    let basis = create_test_state();

    let result = born_probability_kernel(&state, &basis);
    assert!(result.is_ok());

    let p = result.unwrap();
    assert!(
        (0.0..=1.0).contains(&p),
        "Probability must be in [0,1], got {}",
        p
    );
    // Identical states ⇒ P = |⟨ψ|ψ⟩|² = 1 (pins the canonical value, not just the range).
    assert!(
        (p - 1.0).abs() < 1e-9,
        "identical states ⇒ P = 1, got {}",
        p
    );
}

#[test]
fn test_born_probability_kernel_dimension_error() {
    let state = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let basis_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    // A different metric is a MetricMismatch, not any error: the kernel checks
    // the metric before it touches the amplitude.
    let err = born_probability_kernel(&state, &basis_wrong).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::MetricMismatch(_)),
        "expected MetricMismatch, got {err:?}"
    );
}

#[test]
fn test_born_probability_kernel_orthogonal() {
    let state1 = create_test_state(); // |0>

    // Create orthogonal state |1> (e1 component)
    let data2 = vec![
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    let mv2 = CausalMultiVector::new(data2, Metric::Euclidean(3)).unwrap();
    let state2 = HilbertState::<f64>::from_multivector(mv2);

    let result = born_probability_kernel(&state1, &state2);
    assert!(result.is_ok());

    // Orthogonal blades overlap exactly zero; a bound of 0.01 would accept a
    // systematically small but wrong answer.
    let p = result.unwrap();
    assert!(p.abs() < 1e-15, "orthogonal blades ⇒ P = 0, got {}", p);
}

#[test]
fn test_born_probability_is_the_squared_hermitian_overlap() {
    // psi = 0.6 + 0.8i e1 and phi = 0.6i + 0.8 e1, both unit-norm.
    //   <psi|phi> = conj(0.6)(0.6i) + conj(0.8i)(0.8)
    //             = 0.36i - 0.64i = -0.28i,   |.|^2 = 0.0784
    // Two complex components is the point: without the conjugate the sum is
    // 0.36i + 0.64i = 1.0i and the probability comes out 1.0 instead, still
    // inside the kernel's [0,1] acceptance band and so otherwise invisible.
    let psi = state8([
        Complex::new(0.6, 0.0),
        Complex::new(0.0, 0.8),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ]);
    let phi = state8([
        Complex::new(0.0, 0.6),
        Complex::new(0.8, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ]);

    let p = born_probability_kernel(&psi, &phi).expect("both states are unit-norm");
    assert!((p - 0.0784).abs() < 1e-12, "expected 0.0784, got {p}");
}

#[test]
fn test_born_probabilities_over_a_basis_sum_to_one() {
    // psi = 0.6 + 0.8i e1 against the two blades it occupies. The components
    // are unequal, so the split 0.36 / 0.64 also pins which blade carries
    // which amplitude; an index swap would exchange them.
    let psi = state8([
        Complex::new(0.6, 0.0),
        Complex::new(0.0, 0.8),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ]);

    let p_scalar = born_probability_kernel(&psi, &blade(0)).expect("scalar blade");
    let p_e1 = born_probability_kernel(&psi, &blade(1)).expect("e1 blade");
    assert!((p_scalar - 0.36).abs() < 1e-12, "got {p_scalar}");
    assert!((p_e1 - 0.64).abs() < 1e-12, "got {p_e1}");
    assert!((p_scalar + p_e1 - 1.0).abs() < 1e-12, "completeness");

    // The unoccupied blades carry none of it.
    for i in [2usize, 3, 4, 5, 6, 7] {
        let p = born_probability_kernel(&psi, &blade(i)).expect("blade");
        assert!(
            p.abs() < 1e-15,
            "blade {i} should carry no probability, got {p}"
        );
    }
}

#[test]
fn test_born_probability_kernel_nonfinite() {
    // A state with enormous amplitudes makes |<basis|state>|^2 overflow to
    // +inf, tripping the "Born probability is not finite" guard.
    let huge = vec![Complex::new(f64::MAX, 0.0); 8];
    let mv = CausalMultiVector::new(huge, Metric::Euclidean(3)).unwrap();
    let state = HilbertState::<f64>::from_multivector(mv.clone());
    let basis = HilbertState::<f64>::from_multivector(mv);

    let err = born_probability_kernel(&state, &basis).unwrap_err();
    assert_eq!(
        err,
        QuantumError::NonFiniteValue("Born probability is not finite".into())
    );
}

// =============================================================================
// Expectation Value Kernel Tests
// =============================================================================

#[test]
fn test_expectation_value_kernel_valid() {
    let state = create_test_state();
    let operator = create_test_state(); // Use state as simple operator

    // The scalar 1 is the identity of the algebra, so <psi|1|psi> is the squared
    // norm of a unit state: exactly 1, not merely "some Ok value".
    let result = expectation_value_kernel(&state, &operator);
    let v = result.expect("identity operator on a unit state");
    assert!((v - 1.0).abs() < 1e-12, "expected 1, got {v}");
}

#[test]
fn test_expectation_value_of_a_generator_is_twice_the_real_cross_term() {
    // psi = 0.6 + 0.8 e1 and A = e1. Since e1^2 = 1,
    //   A psi = e1(0.6 + 0.8 e1) = 0.8 + 0.6 e1
    //   <psi|A|psi> = conj(0.6)(0.8) + conj(0.8)(0.6) = 2(0.6)(0.8) = 0.96
    // Zero appears nowhere in this: an omitted cross term gives 0.48, a squared
    // amplitude gives 0.5, and the identity operator gives 1.
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

    let v = expectation_value_kernel(&psi, &blade(1)).expect("e1 is Hermitian");
    assert!((v - 0.96).abs() < 1e-12, "expected 0.96, got {v}");

    // e2 and e3 do not touch the occupied blades, so their expectation is zero.
    for i in [2usize, 4] {
        let v = expectation_value_kernel(&psi, &blade(i)).expect("Hermitian generator");
        assert!(v.abs() < 1e-15, "blade {i}: expected 0, got {v}");
    }
}

#[test]
fn test_expectation_value_uses_the_left_action_of_the_operator() {
    // The previous case is blind to operand order: e1 and 0.6 + 0.8 e1 generate a
    // COMMUTATIVE subalgebra, so A psi = psi A there. This state does not commute
    // with e2.
    //
    // psi = (1 + e1 + e2 - e1e2)/2 is a unit eigenvector of e2 with eigenvalue +1:
    //   e2 psi = (e2 + e2e1 + e2e2 - e2e1e2)/2
    //          = (e2 - e1e2 + 1 + e1)/2                 [e2e1 = -e1e2, e2e1e2 = -e1]
    //          = psi
    // so <psi|e2|psi> = <psi|psi> = 1 exactly.
    //
    // The right action gives psi e2 = (1 - e1 + e2 + e1e2)/2, whose overlap with
    // psi is (1 - 1 + 1 - 1)/4 = 0. One is the extreme of the spectrum, the other
    // its midpoint.
    let h = 0.5;
    let psi = state8([
        Complex::new(h, 0.0),
        Complex::new(h, 0.0),
        Complex::new(h, 0.0),
        Complex::new(-h, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ]);

    let v = expectation_value_kernel(&psi, &blade(2)).expect("e2 is Hermitian");
    assert!((v - 1.0).abs() < 1e-12, "expected 1, got {v}");

    // The eigenvector relation the value rests on, checked directly.
    let evolved = apply_gate_kernel(&psi, &blade(2)).expect("e2 gate");
    assert_coeffs_close(
        &evolved,
        [
            Complex::new(h, 0.0),
            Complex::new(h, 0.0),
            Complex::new(h, 0.0),
            Complex::new(-h, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
        ],
    );
}

#[test]
fn test_expectation_value_scales_quadratically_with_the_state() {
    // <(c psi)|A|(c psi)> = |c|^2 <psi|A|psi>. Two scales separate a correct
    // normalization from one that is linear in the state or ignores it.
    let amps = |c: f64| {
        state8([
            Complex::new(0.6 * c, 0.0),
            Complex::new(0.8 * c, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
        ])
    };
    let full = expectation_value_kernel(&amps(1.0), &blade(1)).expect("unit state");
    let half = expectation_value_kernel(&amps(0.5), &blade(1)).expect("half state");
    assert!((full - 0.96).abs() < 1e-12, "got {full}");
    assert!((half - 0.24).abs() < 1e-12, "quarter of {full}, got {half}");

    // And linearly in the operator: <psi|2A|psi> = 2 <psi|A|psi>.
    let mut twice = [Complex::new(0.0, 0.0); 8];
    twice[1] = Complex::new(2.0, 0.0);
    let doubled = expectation_value_kernel(&amps(1.0), &state8(twice)).expect("2 e1");
    assert!((doubled - 2.0 * full).abs() < 1e-12, "got {doubled}");
}

#[test]
fn test_expectation_value_kernel_dimension_error() {
    let state = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let operator_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let err = expectation_value_kernel(&state, &operator_wrong).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::MetricMismatch(_)),
        "expected MetricMismatch, got {err:?}"
    );
}

// =============================================================================
// Apply Gate Kernel Tests
// =============================================================================

#[test]
fn test_apply_gate_kernel_identity() {
    let state = create_test_state();
    let gate = create_test_state(); // the scalar 1, the algebra's identity

    // The identity returns the state unchanged, component for component.
    let result = apply_gate_kernel(&state, &gate).expect("identity gate");
    let mut expected = [Complex::new(0.0, 0.0); 8];
    expected[0] = Complex::new(1.0, 0.0);
    assert_coeffs_close(&result, expected);
}

#[test]
fn test_apply_gate_multiplies_the_gate_from_the_left() {
    // psi = 0.6 + 0.8 e1 and U = e2. Left action:
    //   U psi = e2(0.6 + 0.8 e1) = 0.6 e2 + 0.8 e2 e1 = 0.6 e2 - 0.8 e1e2
    // because e2 e1 = -e1 e2. Blade 2 gets +0.6 and blade 3 gets -0.8.
    //
    // The right action psi U = 0.6 e2 + 0.8 e1 e2 lands +0.8 on blade 3, so the
    // SIGN there is what separates the two operand orders. The old fixture could
    // not: the scalar 1 commutes with everything.
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

    let out = apply_gate_kernel(&psi, &blade(2)).expect("e2 gate");
    let mut expected = [Complex::new(0.0, 0.0); 8];
    expected[2] = Complex::new(0.6, 0.0);
    expected[3] = Complex::new(-0.8, 0.0);
    assert_coeffs_close(&out, expected);

    // A unit blade is unitary here, so the norm survives: 0.6^2 + 0.8^2 = 1.
    let norm_sq: f64 = coeffs(&out).iter().map(|c| c.re * c.re + c.im * c.im).sum();
    assert!((norm_sq - 1.0).abs() < 1e-12, "got {norm_sq}");
}

#[test]
fn test_apply_gate_kernel_dimension_error() {
    let state = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let gate_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let err = apply_gate_kernel(&state, &gate_wrong).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::MetricMismatch(_)),
        "expected MetricMismatch, got {err:?}"
    );
}

#[test]
fn test_apply_gate_kernel_nonfinite() {
    // A gate and state with enormous amplitudes make the geometric product
    // overflow to non-finite components, tripping the "Non-finite component in
    // state after gate application" guard.
    let huge = vec![Complex::new(f64::MAX, 0.0); 8];
    let mv = CausalMultiVector::new(huge, Metric::Euclidean(3)).unwrap();
    let state = HilbertState::<f64>::from_multivector(mv.clone());
    let gate = HilbertState::<f64>::from_multivector(mv);

    let err = apply_gate_kernel(&state, &gate).unwrap_err();
    assert_eq!(
        err,
        QuantumError::NonFiniteValue("Non-finite component in state after gate application".into())
    );
}

// =============================================================================
// Commutator Kernel Tests
// =============================================================================

#[test]
fn test_commutator_kernel_valid() {
    let op_a = create_test_state();
    let op_b = create_test_state();

    // Both operands are the scalar 1, which commutes with everything, so the
    // commutator vanishes identically. Stated as a value rather than as Ok.
    let result = commutator_kernel(&op_a, &op_b).expect("scalar operands");
    assert_coeffs_close(&result, [Complex::new(0.0, 0.0); 8]);
}

#[test]
fn test_commutator_of_orthogonal_generators_is_twice_their_product() {
    // e1 e2 = -e2 e1, so [e1, e2] = e1e2 - e2e1 = 2 e1e2: blade 3 carries +2.
    // This is the case the suite never had — a commutator that does NOT vanish.
    // Against [A,A] = 0 alone, an implementation returning the zero state for
    // every input is indistinguishable from a correct one.
    let out = commutator_kernel(&blade(1), &blade(2)).expect("orthogonal generators");
    let mut expected = [Complex::new(0.0, 0.0); 8];
    expected[3] = Complex::new(2.0, 0.0);
    assert_coeffs_close(&out, expected);
}

#[test]
fn test_commutator_is_antisymmetric() {
    // [B, A] = -[A, B]. A metamorphic relation over the same kernel, so it
    // holds whatever the magnitude, and it fails for any symmetric combination
    // such as the anticommutator AB + BA.
    let ab = commutator_kernel(&blade(1), &blade(2)).expect("e1, e2");
    let ba = commutator_kernel(&blade(2), &blade(1)).expect("e2, e1");
    for (i, (x, y)) in coeffs(&ab).iter().zip(coeffs(&ba).iter()).enumerate() {
        assert!(
            (x.re + y.re).abs() < 1e-12 && (x.im + y.im).abs() < 1e-12,
            "blade {i}: {x:?} is not the negation of {y:?}"
        );
    }
}

#[test]
fn test_commutator_with_the_identity_vanishes_for_a_non_scalar_operand() {
    // [1, e1] = 0 because the scalar is central. Distinct operands, unlike the
    // [A, A] case, so this is the centrality of 1 rather than a tautology.
    let out = commutator_kernel(&blade(0), &blade(1)).expect("scalar and e1");
    assert_coeffs_close(&out, [Complex::new(0.0, 0.0); 8]);
}

#[test]
fn test_commutator_kernel_dimension_error() {
    let op_a = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let op_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let err = commutator_kernel(&op_a, &op_wrong).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::MetricMismatch(_)),
        "expected MetricMismatch, got {err:?}"
    );
}

#[test]
fn test_commutator_kernel_self_is_zero() {
    // e1, not the scalar 1: [A, A] = 0 must hold because A commutes with
    // itself, not because the operand commutes with everything.
    let op_a = blade(1);

    // [A, A] = 0
    let result = commutator_kernel(&op_a, &op_a).unwrap();

    // [A, A] = 0 identically — check ALL components, not just the scalar part,
    // so a non-scalar leak cannot hide.
    assert!(
        result.mv().data().iter().all(|c| c.norm() < 1e-10),
        "Commutator [A,A] must vanish across all components"
    );
}

// =============================================================================
// Fidelity Kernel Tests
// =============================================================================

#[test]
fn test_fidelity_kernel_identical_states() {
    let ideal = create_test_state();
    let actual = create_test_state();

    let result = fidelity_kernel(&ideal, &actual);
    assert!(result.is_ok());

    let f = result.unwrap();
    assert!(
        (0.0..=1.0).contains(&f),
        "Fidelity must be in [0,1], got {}",
        f
    );
    // Identical states ⇒ F = 1.
    assert!(
        (f - 1.0).abs() < 1e-9,
        "identical states ⇒ F = 1, got {}",
        f
    );
}

#[test]
fn test_fidelity_of_distinct_states_is_their_squared_overlap() {
    // The same pair as the Born test: F = |<psi|phi>|^2 = 0.0784. With only the
    // identical-state case, a fidelity that returned a constant 1 passed.
    let psi = state8([
        Complex::new(0.6, 0.0),
        Complex::new(0.0, 0.8),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ]);
    let phi = state8([
        Complex::new(0.0, 0.6),
        Complex::new(0.8, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ]);

    let f = fidelity_kernel(&psi, &phi).expect("unit-norm pair");
    assert!((f - 0.0784).abs() < 1e-12, "expected 0.0784, got {f}");

    // Fidelity is symmetric because |<a|b>| = |<b|a>|, so the delegation's
    // argument order cannot change the value. Pinned rather than assumed.
    let swapped = fidelity_kernel(&phi, &psi).expect("unit-norm pair");
    assert!((f - swapped).abs() < 1e-15, "{f} != {swapped}");
}

// =============================================================================
// Haruna Gate Kernel Tests
// =============================================================================

#[test]
fn test_expectation_value_kernel_nonfinite() {
    // Amplitudes at f64::MAX overflow the geometric products, so the scalar
    // part of adj(psi)·A·psi is not finite. The kernel reports that instead of
    // handing back an infinity.
    let huge = vec![Complex::new(f64::MAX, 0.0); 8];
    let mv = CausalMultiVector::new(huge, Metric::Euclidean(3)).unwrap();
    let state = HilbertState::<f64>::from_multivector(mv.clone());
    let operator = HilbertState::<f64>::from_multivector(mv);

    let err = expectation_value_kernel(&state, &operator).unwrap_err();
    assert_eq!(
        err,
        QuantumError::NonFiniteValue("expectation value is not finite".into())
    );
}

#[test]
fn test_expectation_value_kernel_rejects_non_hermitian_operator() {
    // A = i·1 is anti-Hermitian, and <psi|A|psi> = i for the unit scalar state.
    // The kernel refuses rather than returning the real projection 0, which
    // would be a different observable.
    let state = create_test_state();
    let mut op_data = vec![Complex::new(0.0, 0.0); 8];
    op_data[0] = Complex::new(0.0, 1.0);
    let operator = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(op_data, Metric::Euclidean(3)).unwrap(),
    );

    let err = expectation_value_kernel(&state, &operator).unwrap_err();
    assert_eq!(
        err,
        QuantumError::NonPositiveOperator(
            "expectation value has a non-negligible imaginary part; operator is not Hermitian"
                .into()
        )
    );
}

#[test]
fn test_commutator_kernel_nonfinite() {
    // Amplitudes at f64::MAX overflow both AB and BA, so AB - BA carries
    // non-finite components and the kernel reports that rather than returning
    // a state full of NaN.
    let huge = vec![Complex::new(f64::MAX, 0.0); 8];
    let mv = CausalMultiVector::new(huge, Metric::Euclidean(3)).unwrap();
    let a = HilbertState::<f64>::from_multivector(mv.clone());
    let b = HilbertState::<f64>::from_multivector(mv);

    let err = commutator_kernel(&a, &b).unwrap_err();
    assert_eq!(
        err,
        QuantumError::NonFiniteValue("Non-finite component in commutator result".into())
    );
}
