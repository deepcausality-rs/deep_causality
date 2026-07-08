/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::NormedScalar;
use deep_causality_num::Float106;

fn assert_normed_scalar<T: NormedScalar>() {}

#[test]
fn test_real_fields_are_normed_scalars() {
    // The clean composition `Field + Normed` covers the real fields. `Complex` is covered by the
    // normed-scalar test in `deep_causality_num_complex`.
    assert_normed_scalar::<f32>();
    assert_normed_scalar::<f64>();
    assert_normed_scalar::<Float106>();
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
}
