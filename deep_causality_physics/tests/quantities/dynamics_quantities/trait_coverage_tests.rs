/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Trait coverage: Debug / Clone / Copy / PartialEq / PartialOrd across all dynamics scalars.

use deep_causality_physics::{
    Acceleration, Area, Force, Frequency, Length, Mass, MomentOfInertia, Speed, Torque, Volume,
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
fn test_dynamics_scalars_traits() {
    // Copy semantics: a bitwise copy compares equal to its source.
    let m1 = Mass::<f64>::new(1.0).unwrap();
    let m2 = m1;
    assert_eq!(m1, m2);

    assert_scalar_traits!(Mass<f64>, 1.0, 2.0);
    assert_scalar_traits!(Speed<f64>, 3.0, 4.0);
    assert_scalar_traits!(Acceleration<f64>, -1.0, 1.0);
    assert_scalar_traits!(Force<f64>, -5.0, 5.0);
    assert_scalar_traits!(Torque<f64>, 2.0, 3.0);
    assert_scalar_traits!(Length<f64>, 2.0, 3.0);
    assert_scalar_traits!(Area<f64>, 4.0, 5.0);
    assert_scalar_traits!(Volume<f64>, 1.0, 2.0);
    assert_scalar_traits!(MomentOfInertia<f64>, 0.5, 1.0);
    assert_scalar_traits!(Frequency<f64>, 60.0, 120.0);
}
