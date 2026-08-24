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

// ---------------------------------------------------------------------------
// The `Normed` blanket over `RealField` itself — a real is its own real type,
// its squared modulus is `x²`, and scaling is plain multiplication.
// ---------------------------------------------------------------------------

#[test]
fn test_normed_real_modulus_squared() {
    use deep_causality_algebra::Normed;
    assert_eq!(Normed::modulus_squared(&3.0_f64), 9.0);
    assert_eq!(Normed::modulus_squared(&-4.0_f64), 16.0);
    assert_eq!(Normed::modulus_squared(&0.0_f64), 0.0);
    // f32 goes through the same blanket.
    assert_eq!(Normed::modulus_squared(&2.5_f32), 6.25);
}

#[test]
fn test_normed_real_scale_by_real() {
    use deep_causality_algebra::Normed;
    assert_eq!(Normed::scale_by_real(&3.0_f64, 2.0), 6.0);
    assert_eq!(Normed::scale_by_real(&-1.5_f64, 4.0), -6.0);
    assert_eq!(Normed::scale_by_real(&7.0_f64, 0.0), 0.0);
    assert_eq!(Normed::scale_by_real(&1.5_f32, 2.0), 3.0);
}

#[test]
fn test_normed_real_associated_type_is_self() {
    use deep_causality_algebra::Normed;
    // The associated `Real` of a real field element is the element's own type, which is what lets
    // one bound serve both the real and the complex cases.
    fn modulus<T: Normed>(x: &T) -> T::Real {
        x.modulus_squared()
    }
    let m: f64 = modulus(&5.0_f64);
    assert_eq!(m, 25.0);
}

#[test]
fn test_normed_real_scaling_is_homogeneous() {
    use deep_causality_algebra::Normed;
    // |s·x|² == s²·|x|² for a real scalar.
    let (x, s) = (3.0_f64, 2.0_f64);
    let scaled = Normed::scale_by_real(&x, s);
    assert_eq!(
        Normed::modulus_squared(&scaled),
        s * s * Normed::modulus_squared(&x)
    );
}
