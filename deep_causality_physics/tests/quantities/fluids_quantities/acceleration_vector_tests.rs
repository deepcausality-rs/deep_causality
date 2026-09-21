/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{AccelerationVector, PhysicsErrorEnum};

// =============================================================================
// AccelerationVector — finiteness only
// =============================================================================

#[test]
fn test_acceleration_vector_new_valid() {
    let a = AccelerationVector::<f64>::new([9.81, 0.0, 0.0]).unwrap();
    assert_eq!(a.value(), &[9.81, 0.0, 0.0]);
}

#[test]
fn test_acceleration_vector_rejects_non_finite() {
    assert!(
        matches!(
            AccelerationVector::<f64>::new([f64::NAN, 0.0, 0.0])
                .unwrap_err()
                .0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            AccelerationVector::<f64>::new([0.0, 0.0, f64::INFINITY])
                .unwrap_err()
                .0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_acceleration_vector_default() {
    assert_eq!(AccelerationVector::<f64>::default().into_inner(), [0.0; 3]);
}

#[test]
fn test_acceleration_vector_new_unchecked() {
    let a = AccelerationVector::<f64>::new_unchecked([1.0, 2.0, 3.0]);
    assert_eq!(a.into_inner(), [1.0, 2.0, 3.0]);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_acceleration_vector_traits() {
    let a = AccelerationVector::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    // `assert_eq!(x, x.clone())` is reflexive and holds for a `PartialEq` that always
    // returns true. The inequality discriminates, and comparing the two `Debug`
    // renderings makes `Debug` observable rather than discarded.
    let other = AccelerationVector::<f64>::new([9.0, 8.0, 7.0]).unwrap();
    assert_ne!(a, other, "distinct values must not compare equal");
    assert_ne!(
        format!("{a:?}"),
        format!("{other:?}"),
        "Debug must distinguish distinct values"
    );
}

// =============================================================================
// `From<NewType> for raw` reverse conversions (uncovered before this block).
// These exercise the `impl From<Velocity3<R>> for [R; 3]` style impls that
// turn an invariant-bearing newtype back into its raw representation.
// =============================================================================

#[test]
fn test_acceleration_vector_into_raw_array() {
    let a = AccelerationVector::<f64>::new([7.0, 8.0, 9.0]).unwrap();
    let raw: [f64; 3] = a.into();
    assert_eq!(raw, [7.0, 8.0, 9.0]);
}
