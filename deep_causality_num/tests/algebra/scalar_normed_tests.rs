/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex, Float106, NormedScalar};

fn assert_normed_scalar<T: NormedScalar>() {}

#[test]
fn test_real_and_complex_are_normed_scalars() {
    // The clean composition `Field + Normed` covers the real fields and complex.
    assert_normed_scalar::<f32>();
    assert_normed_scalar::<f64>();
    assert_normed_scalar::<Float106>();
    assert_normed_scalar::<Complex<f64>>();
    // `Dual` is intentionally NOT a `NormedScalar` (not a field, not `Normed`); see the trait docs.
    // Its omission cannot be asserted at compile time without negative bounds.
}

#[test]
fn test_modulus_through_the_bound() {
    // The real modulus is reachable generically through the composed `Normed` supertrait.
    fn modulus_sq<T: NormedScalar>(x: T) -> T::Real {
        x.modulus_squared()
    }
    assert_eq!(modulus_sq(3.0f64), 9.0);
    assert_eq!(modulus_sq(Complex::new(3.0f64, 4.0)), 25.0);
}
