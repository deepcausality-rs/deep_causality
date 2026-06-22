/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{RotationRateTensor, StrainRateTensor, VelocityGradient};

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
    assert!(VelocityGradient::<f64>::new(m).is_err());
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
    let _ = format!("{:?}", a);
}

// =============================================================================
// Property test: any VelocityGradient decomposes as S + Ω
// =============================================================================

#[test]
fn test_velocity_gradient_decomposes_into_strain_and_rotation() {
    // Arbitrary finite gradient
    let g = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    let _grad = VelocityGradient::<f64>::new(g).unwrap();

    // Symmetric part S = 0.5*(G + G^T)
    let mut s = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            s[i][j] = 0.5 * (g[i][j] + g[j][i]);
        }
    }
    // Antisymmetric part Omega = 0.5*(G - G^T)
    let mut omega = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            omega[i][j] = 0.5 * (g[i][j] - g[j][i]);
        }
    }

    let strain = StrainRateTensor::<f64>::new(s).unwrap();
    let rotation = RotationRateTensor::<f64>::new(omega).unwrap();

    // Verify S + Omega == G
    let s_raw: [[f64; 3]; 3] = strain.into();
    let o_raw: [[f64; 3]; 3] = rotation.into();
    for i in 0..3 {
        for j in 0..3 {
            assert!((s_raw[i][j] + o_raw[i][j] - g[i][j]).abs() < 1e-12);
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
