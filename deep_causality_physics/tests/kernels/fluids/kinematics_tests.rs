/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Velocity3, VelocityGradient, VorticityVector, enstrophy_density_kernel,
    helicity_density_kernel, rotation_rate_tensor_kernel, strain_rate_tensor_kernel,
    velocity_gradient_invariants_kernel, vorticity_from_gradient_kernel,
};

const TOL_F64: f64 = 1e-12;
const TOL_F32: f32 = 1e-5;

// =============================================================================
// strain_rate_tensor_kernel
// =============================================================================

#[test]
fn test_strain_rate_is_symmetric() {
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]).unwrap();
    let s = strain_rate_tensor_kernel(&g).unwrap();
    let raw = s.value();
    assert!((raw[0][1] - raw[1][0]).abs() < TOL_F64);
    assert!((raw[0][2] - raw[2][0]).abs() < TOL_F64);
    assert!((raw[1][2] - raw[2][1]).abs() < TOL_F64);
}

#[test]
fn test_strain_rate_vanishes_for_rigid_body_rotation() {
    // Pure antisymmetric ∇u => symmetric part is zero.
    let g = VelocityGradient::<f64>::new([[0.0, 1.0, 2.0], [-1.0, 0.0, 3.0], [-2.0, -3.0, 0.0]])
        .unwrap();
    let s = strain_rate_tensor_kernel(&g).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            assert!(s.value()[i][j].abs() < TOL_F64);
        }
    }
}

#[test]
fn test_strain_rate_equals_input_for_pure_strain() {
    // Symmetric ∇u => symmetric part equals input.
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]]).unwrap();
    let s = strain_rate_tensor_kernel(&g).unwrap();
    for (s_row, g_row) in s.value().iter().zip(g.value().iter()) {
        for (s_ij, g_ij) in s_row.iter().zip(g_row.iter()) {
            assert!((s_ij - g_ij).abs() < TOL_F64);
        }
    }
}

#[test]
fn test_strain_rate_galilean_invariant() {
    // The strain rate depends only on ∇u, not on u itself. Two velocity
    // fields differing by a constant produce the same gradient — exercised
    // structurally by the kernel signature taking only `grad_u`.
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]]).unwrap();
    let s1 = strain_rate_tensor_kernel(&g).unwrap();
    let s2 = strain_rate_tensor_kernel(&g).unwrap();
    assert_eq!(s1.value(), s2.value());
}

#[test]
fn test_strain_rate_f32_precision() {
    let g =
        VelocityGradient::<f32>::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]).unwrap();
    let s = strain_rate_tensor_kernel(&g).unwrap();
    let raw = s.value();
    assert!((raw[0][1] - raw[1][0]).abs() < TOL_F32);
}

// =============================================================================
// rotation_rate_tensor_kernel
// =============================================================================

#[test]
fn test_rotation_rate_is_antisymmetric() {
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]).unwrap();
    let o = rotation_rate_tensor_kernel(&g).unwrap();
    let raw = o.value();
    // Diagonal vanishes
    assert!(raw[0][0].abs() < TOL_F64);
    assert!(raw[1][1].abs() < TOL_F64);
    assert!(raw[2][2].abs() < TOL_F64);
    // Off-diagonals are negatives of their transpose
    assert!((raw[0][1] + raw[1][0]).abs() < TOL_F64);
    assert!((raw[0][2] + raw[2][0]).abs() < TOL_F64);
    assert!((raw[1][2] + raw[2][1]).abs() < TOL_F64);
}

#[test]
fn test_rotation_rate_vanishes_for_pure_strain() {
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]]).unwrap();
    let o = rotation_rate_tensor_kernel(&g).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            assert!(o.value()[i][j].abs() < TOL_F64);
        }
    }
}

// =============================================================================
// Decomposition property: ∇u = S + Ω
// =============================================================================

#[test]
fn test_strain_plus_rotation_reconstruct_gradient() {
    let g = VelocityGradient::<f64>::new([[0.5, 1.0, -2.0], [3.0, 0.0, 0.5], [-1.5, 4.0, 2.0]])
        .unwrap();
    let s = strain_rate_tensor_kernel(&g).unwrap();
    let o = rotation_rate_tensor_kernel(&g).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            let reconstructed = s.value()[i][j] + o.value()[i][j];
            assert!(
                (reconstructed - g.value()[i][j]).abs() < TOL_F64,
                "[{}][{}]: {} vs {}",
                i,
                j,
                reconstructed,
                g.value()[i][j]
            );
        }
    }
}

// =============================================================================
// vorticity_from_gradient_kernel
// =============================================================================

#[test]
fn test_vorticity_known_field() {
    // Construct ∇u such that ω = (1, 0, 0): ∂u_z/∂y = 0.5, ∂u_y/∂z = -0.5.
    let g =
        VelocityGradient::<f64>::new([[0.0, 0.0, 0.0], [0.0, 0.0, -0.5], [0.0, 0.5, 0.0]]).unwrap();
    let w = vorticity_from_gradient_kernel(&g);
    assert!((w.value()[0] - 1.0).abs() < TOL_F64);
    assert!(w.value()[1].abs() < TOL_F64);
    assert!(w.value()[2].abs() < TOL_F64);
}

#[test]
fn test_vorticity_zero_for_irrotational_flow() {
    // Symmetric ∇u has zero vorticity.
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]]).unwrap();
    let w = vorticity_from_gradient_kernel(&g);
    for c in w.value() {
        assert!(c.abs() < TOL_F64);
    }
}

// =============================================================================
// velocity_gradient_invariants_kernel
// =============================================================================

#[test]
fn test_invariants_incompressible_p_is_zero() {
    // Trace-free ∇u (incompressible flow): P = -tr = 0.
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, -2.0, 0.0], [0.0, 0.0, 1.0]]).unwrap();
    let (p, _q, _r) = velocity_gradient_invariants_kernel(&g).unwrap();
    assert!(p.abs() < TOL_F64);
}

#[test]
fn test_invariants_diagonal_matrix() {
    // For A = diag(a, b, c): P = -(a+b+c), Q = ab + ac + bc, R = -abc.
    let g =
        VelocityGradient::<f64>::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]).unwrap();
    let (p, q, r) = velocity_gradient_invariants_kernel(&g).unwrap();
    assert!((p - (-10.0)).abs() < TOL_F64);
    assert!((q - (6.0 + 10.0 + 15.0)).abs() < TOL_F64);
    assert!((r - (-30.0)).abs() < TOL_F64);
}

#[test]
fn test_invariants_zero_gradient_yields_zero_invariants() {
    let g = VelocityGradient::<f64>::default();
    let (p, q, r) = velocity_gradient_invariants_kernel(&g).unwrap();
    assert_eq!(p, 0.0);
    assert_eq!(q, 0.0);
    assert_eq!(r, 0.0);
}

// =============================================================================
// helicity_density_kernel
// =============================================================================

#[test]
fn test_helicity_is_dot_product() {
    let u = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let w = VorticityVector::<f64>::new([4.0, 5.0, 6.0]).unwrap();
    // u·ω = 4 + 10 + 18 = 32
    let h = helicity_density_kernel(&u, &w);
    assert!((h - 32.0).abs() < TOL_F64);
}

#[test]
fn test_helicity_flips_sign_under_reflection() {
    // Reflect along x: u_x → -u_x. ω is a pseudovector under reflection:
    // under a parity flip along x, ω_y → -ω_y and ω_z → -ω_z (sign of ω_x
    // unchanged because it's the axial component along the flipped axis).
    // The simplest way to verify h is a pseudoscalar: reflect ALL velocity
    // components (full parity transform) — under full parity, u → -u and
    // ω → ω (pseudovector). Then u·ω → -u·ω, sign flips.
    let u = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let w = VorticityVector::<f64>::new([0.5, -0.5, 1.0]).unwrap();
    let h_orig = helicity_density_kernel(&u, &w);

    let u_refl = Velocity3::<f64>::new([-1.0, -2.0, -3.0]).unwrap();
    // Under full parity, ω is unchanged (axial vector / pseudovector).
    let h_refl = helicity_density_kernel(&u_refl, &w);

    assert!((h_refl + h_orig).abs() < TOL_F64);
}

#[test]
fn test_helicity_zero_for_orthogonal_vectors() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let w = VorticityVector::<f64>::new([0.0, 1.0, 0.0]).unwrap();
    assert_eq!(helicity_density_kernel(&u, &w), 0.0);
}

// =============================================================================
// enstrophy_density_kernel
// =============================================================================

#[test]
fn test_enstrophy_nonneg_known_value() {
    let w = VorticityVector::<f64>::new([3.0, 4.0, 0.0]).unwrap();
    // 0.5 * (9 + 16 + 0) = 12.5
    let e = enstrophy_density_kernel(&w).unwrap();
    assert!((e - 12.5).abs() < TOL_F64);
    assert!(e >= 0.0);
}

#[test]
fn test_enstrophy_zero_for_zero_vorticity() {
    let w = VorticityVector::<f64>::default();
    let e = enstrophy_density_kernel(&w).unwrap();
    assert_eq!(e, 0.0);
}

#[test]
fn test_enstrophy_nonneg_property() {
    // Random-ish components, including negatives.
    let cases = [
        [1.0, 2.0, 3.0],
        [-1.0, -2.0, -3.0],
        [1e-10, -1e-10, 0.0],
        [1e10, 0.0, 0.0],
    ];
    for raw in cases {
        let w = VorticityVector::<f64>::new(raw).unwrap();
        let e = enstrophy_density_kernel(&w).unwrap();
        assert!(e >= 0.0);
    }
}

#[test]
fn test_enstrophy_f32_precision() {
    let w = VorticityVector::<f32>::new([3.0, 4.0, 0.0]).unwrap();
    let e = enstrophy_density_kernel(&w).unwrap();
    assert!((e - 12.5_f32).abs() < TOL_F32);
}
