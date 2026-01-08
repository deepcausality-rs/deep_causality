/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::QuantumOps;

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
