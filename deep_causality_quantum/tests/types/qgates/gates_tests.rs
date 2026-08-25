/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::QuantumOps;

fn create_complex_mv() -> CausalMultiVector<Complex<f64>> {
    // [1+i, 0, 0, 0, ...]
    let data = vec![
        Complex::new(1.0, 1.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap()
}

#[test]
fn test_dag_hermitian_conjugate() {
    let mv = create_complex_mv();
    let dag = mv.dag();
    // For scalar component Complex(1, 1), dag should be Complex(1, -1) if implicit conjugation
    // OR just reversion.
    // The implementation comment in gates.rs says:
    // "Quantum DAG is Hermitian Conjugate... For now, assuming Reversion is sufficient... Assuming .reversion() exists."
    // Reversion of scalar is self.
    // If it only does reversion without complex conjugation of coefficients, this might be a limitation to verify.
    // However, let's test what the implementation DOES.

    // Actually, physically, DAG should be conjugate transpose.
    // If implementation only calls reversion, let's verify that.

    // Correct Hermitian Conjugate should give conjugate transpose.
    // Reversion of scalar 1+i is 1+i. Conjugate is 1-i.
    assert_eq!(dag.data()[0], Complex::new(1.0, -1.0));
}

#[test]
fn test_bracket_inner_product() {
    let mv1 = create_complex_mv(); // |psi> = 1+i
    let mv2 = create_complex_mv(); // |phi> = 1+i

    // <psi|phi> = (1+i)* (1+i) if dag is just reversion.
    // = 1 + 2i - 1 = 2i.

    // If dag was proper hermitian, it would be (1-i)(1+i) = 2.

    // If dag is proper hermitian:
    // <psi|phi> = (1-i)(1+i) = 1 - i^2 = 1 + 1 = 2.
    let bracket = mv1.bracket(&mv2);
    assert!((bracket.re - 2.0).abs() < 1e-10);
    assert!((bracket.im - 0.0).abs() < 1e-10);
}

#[test]
fn test_normalize() {
    let mv = create_complex_mv();
    // Norm of 1+i is sqrt(2).
    // Normalized should have magnitude 1.
    // Implementation uses `normalize_l2`.

    let normalized = mv.normalize();
    // L2 norm check of result
    // We assume CausalMultiVector implements a norm method or we check coefficients.

    // Check first element magnitude
    let val = normalized.data()[0];
    let mag = (val.re * val.re + val.im * val.im).sqrt();
    assert!((mag - 1.0).abs() < 1e-10);
}

// NOTE on gates.rs (dag fallback) — the `unwrap_or_else` fallback closure of
// `QuantumOps::dag` for `CausalMultiVector<Complex<R>>`. `dag` reverses and
// conjugates the multivector, then calls
// `CausalMultiVector::new(conjugated_data, reverted.metric())`. The conjugated
// data has exactly the same length as the reverted multivector and reuses its
// metric, so the rebuild is always consistent and `new` always returns `Ok`.
// The `Err(_)` fallback (which would rebuild a zeroed multivector) is therefore
// unreachable for any input.

// =============================================================================
// QuantumOps::expectation_value
// =============================================================================

/// Builds a Cl(3,0) multivector from (blade index, coefficient) pairs; every
/// unnamed blade is zero.
fn mv3(entries: &[(usize, Complex<f64>)]) -> CausalMultiVector<Complex<f64>> {
    let mut data = vec![Complex::new(0.0, 0.0); 8];
    for (idx, c) in entries {
        data[*idx] = *c;
    }
    CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap()
}

/// For a scalar-only state c and a real scalar operator a, ⟨ψ|A|ψ⟩ = a·|c|².
/// The conjugation on the bra side is what makes the answer real: without it
/// the product would be a·c² = 6i for c = 1+i and a = 3.
#[test]
fn test_expectation_value_scalar_state_real_operator() {
    let psi = mv3(&[(0, Complex::new(1.0, 1.0))]);
    let op = mv3(&[(0, Complex::new(3.0, 0.0))]);

    let ev = psi.expectation_value(&op);
    assert!((ev.re - 6.0).abs() < 1e-12, "Re<psi|A|psi> = {}", ev.re);
    assert!(ev.im.abs() < 1e-12, "Im<psi|A|psi> = {}", ev.im);
}

/// ⟨e₁|(2 + e₁)|e₁⟩ = 2 in Cl(3,0). The operator's scalar part contributes
/// 2·e₁e₁ = 2 to the scalar projection, and its vector part contributes
/// e₁·e₁e₁ = e₁, which sits in grade 1 and is projected away.
#[test]
fn test_expectation_value_vector_state_mixed_grade_operator() {
    let psi = mv3(&[(1, Complex::new(1.0, 0.0))]);
    let op = mv3(&[(0, Complex::new(2.0, 0.0)), (1, Complex::new(1.0, 0.0))]);

    let ev = psi.expectation_value(&op);
    assert!((ev.re - 2.0).abs() < 1e-12, "Re<e1|A|e1> = {}", ev.re);
    assert!(ev.im.abs() < 1e-12, "Im<e1|A|e1> = {}", ev.im);
}

/// The trait method returns ⟨ψ|A|ψ⟩ as a complex number. For the
/// anti-Hermitian A = i·1 on the unit scalar state the value is i, so the
/// imaginary part is carried through rather than screened or discarded.
#[test]
fn test_expectation_value_carries_imaginary_part_of_anti_hermitian_operator() {
    let psi = mv3(&[(0, Complex::new(1.0, 0.0))]);
    let op = mv3(&[(0, Complex::new(0.0, 1.0))]);

    let ev = psi.expectation_value(&op);
    assert!(ev.re.abs() < 1e-12, "Re<psi|iI|psi> = {}", ev.re);
    assert!((ev.im - 1.0).abs() < 1e-12, "Im<psi|iI|psi> = {}", ev.im);
}
