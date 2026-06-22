/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, RotationRateTensor};

// =============================================================================
// RotationRateTensor — antisymmetric (Ω_ij == -Ω_ji, Ω_ii == 0)
// =============================================================================

#[test]
fn test_rotation_rate_tensor_new_valid_antisymmetric() {
    let omega = [[0.0, 1.0, 2.0], [-1.0, 0.0, 3.0], [-2.0, -3.0, 0.0]];
    let t = RotationRateTensor::<f64>::new(omega).unwrap();
    assert_eq!(t.value(), &omega);
}

#[test]
fn test_rotation_rate_tensor_rejects_nonzero_diagonal() {
    let omega = [[1.0, 1.0, 2.0], [-1.0, 0.0, 3.0], [-2.0, -3.0, 0.0]];
    let r = RotationRateTensor::<f64>::new(omega);
    assert!(r.is_err());
    match &r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("diagonal") || msg.contains("antisymmetric"))
        }
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_rotation_rate_tensor_rejects_non_antisymmetric_off_diagonal() {
    // Ω_01 = 1.0 but Ω_10 = 1.0 (should be -1.0)
    let omega = [[0.0, 1.0, 2.0], [1.0, 0.0, 3.0], [-2.0, -3.0, 0.0]];
    let r = RotationRateTensor::<f64>::new(omega);
    assert!(r.is_err());
    match &r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("antisymmetric")),
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_rotation_rate_tensor_rejects_non_finite() {
    let omega = [
        [0.0, f64::INFINITY, 0.0],
        [-f64::INFINITY, 0.0, 0.0],
        [0.0, 0.0, 0.0],
    ];
    assert!(RotationRateTensor::<f64>::new(omega).is_err());
}

#[test]
fn test_rotation_rate_tensor_default_is_zero() {
    assert_eq!(
        RotationRateTensor::<f64>::default().into_inner(),
        [[0.0; 3]; 3]
    );
}

#[test]
fn test_rotation_rate_tensor_new_unchecked_bypasses_check() {
    let omega = [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let t = RotationRateTensor::<f64>::new_unchecked(omega);
    assert_eq!(t.into_inner(), omega);
}

#[test]
fn test_rotation_rate_tensor_from_self_to_raw() {
    let omega = [[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let t = RotationRateTensor::<f64>::new(omega).unwrap();
    let raw: [[f64; 3]; 3] = t.into();
    assert_eq!(raw, omega);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_rotation_rate_tensor_traits() {
    let omega = [[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let a = RotationRateTensor::<f64>::new(omega).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    let _ = format!("{:?}", a);
}

// =============================================================================
// `From<NewType> for raw` reverse conversions (uncovered before this block).
// These exercise the `impl From<Velocity3<R>> for [R; 3]` style impls that
// turn an invariant-bearing newtype back into its raw representation.
// =============================================================================

#[test]
fn test_rotation_rate_tensor_into_raw_matrix() {
    let m = [[0.0, 1.0, 2.0], [-1.0, 0.0, 3.0], [-2.0, -3.0, 0.0]];
    let o = RotationRateTensor::<f64>::new(m).unwrap();
    let raw: [[f64; 3]; 3] = o.into();
    assert_eq!(raw, m);
}
