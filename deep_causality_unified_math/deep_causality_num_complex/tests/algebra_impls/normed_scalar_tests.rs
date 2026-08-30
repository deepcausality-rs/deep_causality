/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{Normed, NormedScalar};
use deep_causality_num_complex::Complex;

fn assert_normed_scalar<T: NormedScalar>() {}

#[test]
fn test_complex_is_normed_scalar() {
    // The clean composition `Field + Normed` covers complex.
    assert_normed_scalar::<Complex<f64>>();
}

#[test]
fn test_modulus_through_the_bound() {
    // The real modulus is reachable generically through the composed `Normed` supertrait.
    fn modulus_sq<T: NormedScalar>(x: T) -> T::Real {
        x.modulus_squared()
    }
    assert_eq!(modulus_sq(Complex::new(3.0f64, 4.0)), 25.0);
}

#[test]
fn test_complex_normed_impl() {
    // Exercise the `Normed` impl directly: squared modulus and real scaling.
    let z = Complex::new(3.0f64, 4.0);
    assert_eq!(Normed::modulus_squared(&z), 25.0);
    let s = z.scale_by_real(2.0);
    assert_eq!(s.re(), 6.0);
    assert_eq!(s.im(), 8.0);
}
