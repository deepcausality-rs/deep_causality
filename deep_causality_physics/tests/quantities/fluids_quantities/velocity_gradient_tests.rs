/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    PhysicsErrorEnum, RotationRateTensor, StrainRateTensor, VelocityGradient,
    rotation_rate_tensor_kernel, strain_rate_tensor_kernel,
};

// =============================================================================
// VelocityGradient — Jacobian convention pinned at construction
// =============================================================================

#[test]
fn test_velocity_gradient_new_valid() {
    let m = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    let g = VelocityGradient::<f64>::new(m).unwrap();
    assert_eq!(g.value(), &m);
}

#[test]
fn test_velocity_gradient_rejects_non_finite() {
    let mut m = [[0.0; 3]; 3];
    m[1][2] = f64::NAN;
    assert!(
        matches!(
            VelocityGradient::<f64>::new(m).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_velocity_gradient_default_is_zero() {
    assert_eq!(
        VelocityGradient::<f64>::default().into_inner(),
        [[0.0; 3]; 3]
    );
}

#[test]
fn test_velocity_gradient_new_unchecked() {
    let m = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    assert_eq!(VelocityGradient::<f64>::new_unchecked(m).into_inner(), m);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_velocity_gradient_traits() {
    let m = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let a = VelocityGradient::<f64>::new(m).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    // `assert_eq!(x, x.clone())` is reflexive and holds for a `PartialEq` that always
    // returns true. The inequality discriminates, and comparing the two `Debug`
    // renderings makes `Debug` observable rather than discarded.
    let other =
        VelocityGradient::<f64>::new([[9.0, 0.0, 0.0], [0.0, 9.0, 0.0], [0.0, 0.0, 9.0]]).unwrap();
    assert_ne!(a, other, "distinct values must not compare equal");
    assert_ne!(
        format!("{a:?}"),
        format!("{other:?}"),
        "Debug must distinguish distinct values"
    );
}

// =============================================================================
// Property test: any VelocityGradient decomposes as S + Ω
// =============================================================================

#[test]
fn test_velocity_gradient_decomposes_into_strain_and_rotation() {
    // The gradient used to be constructed into `_grad` and discarded: the decomposition was
    // computed from the raw array in this test and then checked against that same array, so the
    // `VelocityGradient` newtype played no part and the assertion was arithmetic the test had
    // just performed itself.
    //
    // The split now goes through the kernels that own it, and the reconstruction is checked
    // against the gradient read back out of the newtype.
    let g = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    let grad = VelocityGradient::<f64>::new(g).unwrap();

    let strain: StrainRateTensor<f64> = strain_rate_tensor_kernel(&grad).unwrap();
    let rotation: RotationRateTensor<f64> = rotation_rate_tensor_kernel(&grad).unwrap();

    // The newtype must hand back what it was given, so the target of the reconstruction is the
    // gradient itself rather than the local array.
    let g_raw: [[f64; 3]; 3] = grad.into();
    assert_eq!(g_raw, g, "the newtype must carry the matrix verbatim");

    let s_raw: [[f64; 3]; 3] = strain.into();
    let o_raw: [[f64; 3]; 3] = rotation.into();
    for i in 0..3 {
        for j in 0..3 {
            assert!(
                (s_raw[i][j] + o_raw[i][j] - g_raw[i][j]).abs() < 1e-12,
                "[{i}][{j}]: S + Omega = {}, expected {}",
                s_raw[i][j] + o_raw[i][j],
                g_raw[i][j]
            );
            // S is symmetric and Omega antisymmetric; neither holds for an arbitrary split.
            assert!((s_raw[i][j] - s_raw[j][i]).abs() < 1e-12, "S not symmetric");
            assert!(
                (o_raw[i][j] + o_raw[j][i]).abs() < 1e-12,
                "Omega not antisymmetric"
            );
        }
    }
}

// =============================================================================
// `From<NewType> for raw` reverse conversions (uncovered before this block).
// These exercise the `impl From<Velocity3<R>> for [R; 3]` style impls that
// turn an invariant-bearing newtype back into its raw representation.
// =============================================================================

#[test]
fn test_velocity_gradient_into_raw_matrix() {
    let m = [[1.0, 0.0, 0.0], [0.0, -2.0, 0.0], [0.0, 0.0, 0.5]];
    let g = VelocityGradient::<f64>::new(m).unwrap();
    let raw: [[f64; 3]; 3] = g.into();
    assert_eq!(raw, m);
}
