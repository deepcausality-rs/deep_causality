/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::ConjugateScalar;
use deep_causality_num_complex::Complex;

fn assert_conjugate_scalar<T: ConjugateScalar>() {}

#[test]
fn test_complex_is_conjugate_scalar() {
    assert_conjugate_scalar::<Complex<f64>>();
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
