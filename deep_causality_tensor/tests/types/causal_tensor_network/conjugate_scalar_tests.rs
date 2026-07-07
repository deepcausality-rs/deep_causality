/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::ConjugateScalar;
use deep_causality_num::Float106;
use deep_causality_num_complex::Complex;
use deep_causality_num_dual::Dual;

fn assert_conj<T: ConjugateScalar>() {}

#[test]
fn test_all_scalar_families_are_conjugate_scalars() {
    // Reals, dual (AD), and complex all satisfy the one bridge bound.
    assert_conj::<f32>();
    assert_conj::<f64>();
    assert_conj::<Float106>();
    assert_conj::<Dual<f64>>();
    assert_conj::<Complex<f64>>();
}

#[test]
fn test_conjugate_and_modulus_semantics() {
    // Real: conjugate is identity, modulus² is x².
    assert_eq!(ConjugateScalar::conjugate(&3.0f64), 3.0);
    assert_eq!((3.0f64).modulus_squared(), 9.0);
    assert_eq!(<f64 as ConjugateScalar>::from_real(2.5), 2.5);

    // Complex: conjugate flips the imaginary part, modulus² = re² + im².
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
