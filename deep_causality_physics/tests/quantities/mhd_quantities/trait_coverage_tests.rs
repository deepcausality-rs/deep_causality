/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AlfvenSpeed, Conductivity, DebyeLength, Diffusivity, LarmorRadius, MagneticPressure,
    PlasmaBeta, PlasmaFrequency,
};

#[test]
fn test_mhd_scalars_into_f64() {
    let v: f64 = AlfvenSpeed::<f64>::new(100.0).unwrap().into();
    assert_eq!(v, 100.0);

    let v: f64 = PlasmaBeta::<f64>::new(0.5).unwrap().into();
    assert_eq!(v, 0.5);

    let v: f64 = MagneticPressure::<f64>::new(1000.0).unwrap().into();
    assert_eq!(v, 1000.0);

    let v: f64 = LarmorRadius::<f64>::new(1.0).unwrap().into();
    assert_eq!(v, 1.0);

    let v: f64 = DebyeLength::<f64>::new(1e-6).unwrap().into();
    assert_eq!(v, 1e-6);

    let v: f64 = PlasmaFrequency::<f64>::new(1e9).unwrap().into();
    assert_eq!(v, 1e9);

    let v: f64 = Conductivity::<f64>::new(1e7).unwrap().into();
    assert_eq!(v, 1e7);

    let v: f64 = Diffusivity::<f64>::new(1.0).unwrap().into();
    assert_eq!(v, 1.0);
}

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
fn test_mhd_scalars_traits() {
    // Copy semantics: a bitwise copy compares equal to its source.
    let a = AlfvenSpeed::<f64>::new(1.0).unwrap();
    let copied = a;
    assert_eq!(a, copied);

    assert_scalar_traits!(AlfvenSpeed<f64>, 1.0, 2.0);
    assert_scalar_traits!(PlasmaBeta<f64>, 0.5, 1.0);
    assert_scalar_traits!(MagneticPressure<f64>, 100.0, 200.0);
    assert_scalar_traits!(LarmorRadius<f64>, 1.0, 2.0);
    assert_scalar_traits!(DebyeLength<f64>, 1.0, 2.0);
    assert_scalar_traits!(PlasmaFrequency<f64>, 1.0, 2.0);
    assert_scalar_traits!(Conductivity<f64>, 1.0, 2.0);
    assert_scalar_traits!(Diffusivity<f64>, 1.0, 2.0);
}
