/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::ConjugateScalar;
use deep_causality_num::Float106;

fn assert_conjugate_scalar<T: ConjugateScalar>() {}

#[test]
fn test_real_families_are_conjugate_scalars() {
    // The real fields all satisfy the one bridge bound. `Dual` and `Complex` are covered by the
    // conjugate-scalar tests in their own crates.
    assert_conjugate_scalar::<f32>();
    assert_conjugate_scalar::<f64>();
    assert_conjugate_scalar::<Float106>();
}

#[test]
fn test_real_semantics() {
    // Conjugation is the identity and the modulus is `x²`.
    assert_eq!(ConjugateScalar::conjugate(&3.0f64), 3.0);
    assert_eq!((3.0f64).modulus_squared(), 9.0);
    assert_eq!((3.0f64).real_part(), 3.0);
    assert_eq!(<f64 as ConjugateScalar>::from_real(2.5), 2.5);
}
