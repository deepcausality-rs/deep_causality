/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    VelocityGradient, delta_criterion_kernel, lambda2_kernel, q_criterion_kernel,
    rotation_rate_tensor_kernel, strain_rate_tensor_kernel, swirling_strength_kernel,
};

const TOL_F64: f64 = 1e-10;
const TOL_F32: f32 = 1e-4;

// =============================================================================
// q_criterion_kernel
// =============================================================================

#[test]
fn test_q_criterion_identity_q_plus_half_s2_minus_half_o2_eq_zero() {
    // Spec scenario: Q + 0.5·‖S‖² − 0.5·‖Ω‖² == 0  for any ∇u.
    let g = VelocityGradient::<f64>::new([[0.5, 1.0, -2.0], [3.0, 0.0, 0.5], [-1.5, 4.0, 2.0]])
        .unwrap();
    let q = q_criterion_kernel(&g).unwrap();
    let s = strain_rate_tensor_kernel(&g).unwrap();
    let o = rotation_rate_tensor_kernel(&g).unwrap();
    let s_norm_sq: f64 = s.value().iter().flatten().map(|x| x * x).sum();
    let o_norm_sq: f64 = o.value().iter().flatten().map(|x| x * x).sum();
    let residual = q + 0.5 * s_norm_sq - 0.5 * o_norm_sq;
    assert!(residual.abs() < TOL_F64);
}

#[test]
fn test_q_criterion_positive_for_rigid_body_rotation() {
    // Pure rotation in the x-y plane: S = 0, only Ω ≠ 0.
    let omega = 2.0;
    let g = VelocityGradient::<f64>::new([[0.0, -omega, 0.0], [omega, 0.0, 0.0], [0.0, 0.0, 0.0]])
        .unwrap();
    // ‖Ω‖² = 2ω², ‖S‖² = 0 ⇒ Q = ω² > 0.
    let q = q_criterion_kernel(&g).unwrap();
    assert!((q - omega * omega).abs() < TOL_F64);
}

#[test]
fn test_q_criterion_negative_for_pure_strain() {
    // Pure symmetric ∇u: Ω = 0, only S ≠ 0 ⇒ Q < 0.
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 0.0], [2.0, -1.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let q = q_criterion_kernel(&g).unwrap();
    assert!(q < 0.0);
}

#[test]
fn test_q_criterion_zero_for_zero_gradient() {
    let g = VelocityGradient::<f64>::default();
    assert_eq!(q_criterion_kernel(&g).unwrap(), 0.0);
}

#[test]
fn test_q_criterion_identity_f32() {
    let g = VelocityGradient::<f32>::new([[0.5, 1.0, -2.0], [3.0, 0.0, 0.5], [-1.5, 4.0, 2.0]])
        .unwrap();
    let q = q_criterion_kernel(&g).unwrap();
    let s = strain_rate_tensor_kernel(&g).unwrap();
    let o = rotation_rate_tensor_kernel(&g).unwrap();
    let s_norm_sq: f32 = s.value().iter().flatten().map(|x| x * x).sum();
    let o_norm_sq: f32 = o.value().iter().flatten().map(|x| x * x).sum();
    let residual = q + 0.5 * s_norm_sq - 0.5 * o_norm_sq;
    assert!(residual.abs() < TOL_F32);
}

// =============================================================================
// delta_criterion_kernel
// =============================================================================

#[test]
fn test_delta_criterion_positive_for_rotational_flow() {
    // Same rigid-body rotation as above. Eigenvalues of ∇u are 0, ±iω
    // → complex pair exists → Δ > 0.
    let omega = 1.5;
    let g = VelocityGradient::<f64>::new([[0.0, -omega, 0.0], [omega, 0.0, 0.0], [0.0, 0.0, 0.0]])
        .unwrap();
    let delta = delta_criterion_kernel(&g).unwrap();
    assert!(delta > 0.0);
}

#[test]
fn test_delta_criterion_zero_for_zero_gradient() {
    let g = VelocityGradient::<f64>::default();
    assert_eq!(delta_criterion_kernel(&g).unwrap(), 0.0);
}

#[test]
fn test_delta_criterion_negative_for_three_distinct_real_eigenvalues() {
    // Diagonal ∇u with three distinct real eigenvalues (1, 2, 3). The
    // depressed-cubic discriminant is < 0 ⇒ three distinct real roots,
    // i.e. node/saddle topology, no swirl.
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]]).unwrap();
    let delta = delta_criterion_kernel(&g).unwrap();
    assert!(
        delta < 0.0,
        "expected Δ < 0 for distinct real eigenvalues, got {}",
        delta
    );
}

#[test]
fn test_delta_criterion_zero_for_repeated_real_eigenvalues() {
    // diag(1, -2, 1) has repeated eigenvalue 1 ⇒ Δ = 0 (discriminant boundary).
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, -2.0, 0.0], [0.0, 0.0, 1.0]]).unwrap();
    let delta = delta_criterion_kernel(&g).unwrap();
    assert!(delta.abs() < TOL_F64);
}

// =============================================================================
// lambda2_kernel
// =============================================================================

#[test]
fn test_lambda2_negative_for_rigid_body_rotation() {
    // Spec scenario: λ₂ < 0 inside a vortex tube. Rigid-body rotation
    // ∇u = [[0,-ω,0],[ω,0,0],[0,0,0]] has S = 0 and Ω with
    // Ω² = diag(-ω², -ω², 0). M = S² + Ω² = Ω². Eigenvalues sorted
    // descending: [0, -ω², -ω²]. λ₂ = -ω² < 0.
    let omega = 1.0;
    let g = VelocityGradient::<f64>::new([[0.0, -omega, 0.0], [omega, 0.0, 0.0], [0.0, 0.0, 0.0]])
        .unwrap();
    let l2 = lambda2_kernel(&g).unwrap();
    assert!((l2 - (-omega * omega)).abs() < TOL_F64);
    assert!(l2 < 0.0);
}

#[test]
fn test_lambda2_zero_for_zero_gradient() {
    let g = VelocityGradient::<f64>::default();
    assert_eq!(lambda2_kernel(&g).unwrap(), 0.0);
}

#[test]
fn test_lambda2_positive_for_pure_extension() {
    // Pure isotropic extension: S = I, Ω = 0. M = I² + 0 = I. All eigenvalues = 1.
    // Middle eigenvalue = 1 > 0 (no vortex).
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]).unwrap();
    let l2 = lambda2_kernel(&g).unwrap();
    assert!((l2 - 1.0).abs() < TOL_F64);
}

#[test]
fn test_lambda2_diagonal_strain() {
    // ∇u = diag(2, 3, 5) ⇒ S = ∇u, Ω = 0. M = S² = diag(4, 9, 25).
    // Sorted descending: [25, 9, 4]. λ₂ = 9.
    let g =
        VelocityGradient::<f64>::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]).unwrap();
    let l2 = lambda2_kernel(&g).unwrap();
    assert!((l2 - 9.0).abs() < TOL_F64);
}

// =============================================================================
// swirling_strength_kernel
// =============================================================================

#[test]
fn test_swirling_strength_zero_for_irrotational_flow() {
    // Spec scenario: swirling strength vanishes when vorticity is zero.
    // Symmetric ∇u ⇒ vorticity = 0, and a real symmetric matrix has all
    // real eigenvalues ⇒ λ_ci = 0.
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]]).unwrap();
    let lci = swirling_strength_kernel(&g).unwrap();
    assert!(lci.abs() < TOL_F64);
}

#[test]
fn test_swirling_strength_for_rigid_body_rotation() {
    // Eigenvalues of ∇u = [[0,-ω,0],[ω,0,0],[0,0,0]] are 0, ±iω.
    // λ_ci = |Im(λ)| = ω.
    let omega = 1.5;
    let g = VelocityGradient::<f64>::new([[0.0, -omega, 0.0], [omega, 0.0, 0.0], [0.0, 0.0, 0.0]])
        .unwrap();
    let lci = swirling_strength_kernel(&g).unwrap();
    assert!((lci - omega).abs() < TOL_F64);
}

#[test]
fn test_swirling_strength_zero_for_zero_gradient() {
    let g = VelocityGradient::<f64>::default();
    assert_eq!(swirling_strength_kernel(&g).unwrap(), 0.0);
}

#[test]
fn test_swirling_strength_nonneg() {
    for g_raw in [
        [[0.0, -2.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
        [[0.5, 1.0, -2.0], [3.0, 0.0, 0.5], [-1.5, 4.0, 2.0]],
        [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]],
    ] {
        let g = VelocityGradient::<f64>::new(g_raw).unwrap();
        let lci = swirling_strength_kernel(&g).unwrap();
        assert!(lci >= 0.0);
    }
}

// =============================================================================
// Cross-checks
// =============================================================================

#[test]
fn test_swirling_strength_consistent_with_delta_sign() {
    // Δ > 0 implies complex eigenvalues implies λ_ci > 0; Δ < 0 implies
    // λ_ci = 0 (all real eigenvalues).
    let cases: [[[f64; 3]; 3]; 3] = [
        [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]], // rotation: Δ > 0
        [[1.0, 0.0, 0.0], [0.0, -2.0, 0.0], [0.0, 0.0, 1.0]], // distinct real: Δ < 0
        [[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]],  // symmetric: Δ ≤ 0
    ];
    for g_raw in cases {
        let g = VelocityGradient::<f64>::new(g_raw).unwrap();
        let delta = delta_criterion_kernel(&g).unwrap();
        let lci = swirling_strength_kernel(&g).unwrap();
        if delta > TOL_F64 {
            assert!(lci > 0.0, "delta={} lci={}", delta, lci);
        } else if delta < -TOL_F64 {
            assert!(lci.abs() < TOL_F64, "delta={} lci={}", delta, lci);
        }
    }
}
