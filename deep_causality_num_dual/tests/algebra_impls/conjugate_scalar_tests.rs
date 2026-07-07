/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::ConjugateScalar;
use deep_causality_num_dual::Dual;

fn assert_conjugate_scalar<T: ConjugateScalar>() {}

#[test]
fn test_dual_is_conjugate_scalar() {
    assert_conjugate_scalar::<Dual<f64>>();
}

#[test]
fn test_dual_modulus_threads_derivative() {
    // Dual conjugation is the identity; the modulus carries the derivative (its `Real` is `Dual`),
    // which is exactly what lets singular values differentiate.
    let d = Dual::new(2.0f64, 1.0); // value 2, derivative seed 1
    assert_eq!(ConjugateScalar::conjugate(&d), d);
    let m = d.modulus_squared(); // (2 + ε)² = 4 + 4ε
    assert_eq!(m.value(), 4.0);
    assert_eq!(m.derivative(), 4.0);
    assert_eq!(<Dual<f64> as ConjugateScalar>::from_real(d), d);
}
