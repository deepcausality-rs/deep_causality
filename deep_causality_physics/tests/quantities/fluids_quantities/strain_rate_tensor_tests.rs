/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, StrainRateTensor};

// =============================================================================
// StrainRateTensor — symmetric (S_ij == S_ji)
// =============================================================================

#[test]
fn test_strain_rate_tensor_new_valid_symmetric() {
    let s = [[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let t = StrainRateTensor::<f64>::new(s).unwrap();
    assert_eq!(t.value(), &s);
}

#[test]
fn test_strain_rate_tensor_rejects_asymmetric() {
    // S_01 = 2.0 but S_10 = 9.0 — clearly asymmetric
    let s = [[1.0, 2.0, 3.0], [9.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let r = StrainRateTensor::<f64>::new(s);
    match &r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("symmetric")),
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_strain_rate_tensor_rejects_non_finite() {
    let s = [[f64::NAN, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    assert!(
        matches!(
            StrainRateTensor::<f64>::new(s).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_strain_rate_tensor_default_is_zero() {
    assert_eq!(
        StrainRateTensor::<f64>::default().into_inner(),
        [[0.0; 3]; 3]
    );
}

#[test]
fn test_strain_rate_tensor_new_unchecked_bypasses_check() {
    // Asymmetric matrix — accepted via new_unchecked, would be rejected by new.
    let s = [[1.0, 2.0, 0.0], [9.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let t = StrainRateTensor::<f64>::new_unchecked(s);
    assert_eq!(t.into_inner(), s);
}

#[test]
fn test_strain_rate_tensor_from_self_to_raw_drops_invariant() {
    let s = [[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let t = StrainRateTensor::<f64>::new(s).unwrap();
    let raw: [[f64; 3]; 3] = t.into();
    assert_eq!(raw, s);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_strain_rate_tensor_traits() {
    let s = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let a = StrainRateTensor::<f64>::new(s).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    // `assert_eq!(x, x.clone())` is reflexive and holds for a `PartialEq` that always
    // returns true. The inequality discriminates, and comparing the two `Debug`
    // renderings makes `Debug` observable rather than discarded.
    let other =
        StrainRateTensor::<f64>::new([[9.0, 0.0, 0.0], [0.0, 9.0, 0.0], [0.0, 0.0, 9.0]]).unwrap();
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
fn test_strain_rate_tensor_into_raw_matrix() {
    let m = [[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let s = StrainRateTensor::<f64>::new(m).unwrap();
    let raw: [[f64; 3]; 3] = s.into();
    assert_eq!(raw, m);
}
