/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex, ConjugateScalar, Dual, Float106};

fn assert_conjugate_scalar<T: ConjugateScalar>() {}

#[test]
fn test_all_families_are_conjugate_scalars() {
    // Reals, dual (AD), and complex all satisfy the one bridge bound.
    assert_conjugate_scalar::<f32>();
    assert_conjugate_scalar::<f64>();
    assert_conjugate_scalar::<Float106>();
    assert_conjugate_scalar::<Dual<f64>>();
    assert_conjugate_scalar::<Complex<f64>>();
}

#[test]
fn test_real_semantics() {
    // Conjugation is the identity and the modulus is `x²`.
    assert_eq!(ConjugateScalar::conjugate(&3.0f64), 3.0);
    assert_eq!((3.0f64).modulus_squared(), 9.0);
    assert_eq!((3.0f64).real_part(), 3.0);
    assert_eq!(<f64 as ConjugateScalar>::from_real(2.5), 2.5);
}

#[test]
fn test_complex_semantics() {
    // Conjugation flips the imaginary part; the modulus is `re² + im²`; magnitudes are real.
    let z = Complex::new(3.0f64, 4.0);
    let zc = ConjugateScalar::conjugate(&z);
    assert_eq!(zc.re(), 3.0);
    assert_eq!(zc.im(), -4.0);
    assert_eq!(z.modulus_squared(), 25.0);
    assert_eq!(z.real_part(), 3.0);
    assert_eq!(
        <Complex<f64> as ConjugateScalar>::from_real(7.0),
        Complex::new(7.0, 0.0)
    );
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
