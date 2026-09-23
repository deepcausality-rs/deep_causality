/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    BeamWaist, FocalLength, NumericalAperture, OpticalPower, RayAngle, RayHeight, Wavelength,
};

/// The trait contract for a newtype over a scalar, checked against two *distinct* values.
///
/// `assert_eq!(a, a.clone())` is reflexive: it holds for any derived `PartialEq` and for a broken
/// one that always returns true, so it cannot fail. The inequality is what discriminates, and
/// comparing the two `Debug` renderings is what makes `Debug` observable instead of discarded.
macro_rules! assert_scalar_traits {
    ($ty:ty, $small:expr, $large:expr) => {{
        let a = <$ty>::new($small).unwrap();
        let b = <$ty>::new($large).unwrap();
        assert_eq!(a, a.clone(), "clone must preserve equality");
        assert_ne!(a, b, "distinct values must not compare equal");
        assert!(a < b, "ordering must follow the wrapped value");
        assert_ne!(
            format!("{:?}", a),
            format!("{:?}", b),
            "Debug must distinguish distinct values"
        );
    }};
}

#[test]
fn test_photonics_scalars_traits() {
    assert_scalar_traits!(FocalLength<f64>, 1.0, 2.0);
    assert_scalar_traits!(OpticalPower<f64>, 2.0, 3.0);
    assert_scalar_traits!(Wavelength<f64>, 500e-9, 600e-9);
    assert_scalar_traits!(NumericalAperture<f64>, 0.5, 0.75);
    assert_scalar_traits!(BeamWaist<f64>, 1e-6, 2e-6);
    assert_scalar_traits!(RayHeight<f64>, 0.01, 0.02);
    assert_scalar_traits!(RayAngle<f64>, 0.1, 0.2);
}
