/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::VorticityVector;

// =============================================================================
// VorticityVector — finiteness only
// =============================================================================

#[test]
fn test_vorticity_vector_new_valid() {
    let w = VorticityVector::<f64>::new([0.5, -0.5, 1.0]).unwrap();
    assert_eq!(w.value(), &[0.5, -0.5, 1.0]);
}

#[test]
fn test_vorticity_vector_rejects_non_finite() {
    assert!(VorticityVector::<f64>::new([f64::NAN, 0.0, 0.0]).is_err());
    assert!(VorticityVector::<f64>::new([f64::INFINITY, 0.0, 0.0]).is_err());
}

#[test]
fn test_vorticity_vector_default() {
    assert_eq!(VorticityVector::<f64>::default().into_inner(), [0.0; 3]);
}

#[test]
fn test_vorticity_vector_new_unchecked() {
    let w = VorticityVector::<f64>::new_unchecked([1.0, 2.0, 3.0]);
    assert_eq!(w.into_inner(), [1.0, 2.0, 3.0]);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_vorticity_vector_traits() {
    let a = VorticityVector::<f64>::new([0.1, 0.2, 0.3]).unwrap();
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
fn test_vorticity_vector_into_raw_array() {
    let w = VorticityVector::<f64>::new([4.0, 5.0, 6.0]).unwrap();
    let raw: [f64; 3] = w.into();
    assert_eq!(raw, [4.0, 5.0, 6.0]);
}
