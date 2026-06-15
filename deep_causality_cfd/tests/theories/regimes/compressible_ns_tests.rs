/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{
    compressible_ns_continuity_rhs_kernel, compressible_ns_energy_rhs_kernel,
    compressible_ns_momentum_rhs_kernel, incompressible_ns_rhs_kernel,
};
use deep_causality_physics::{
    AccelerationVector, Density, KinematicViscosity, StrainRateTensor, Velocity3, VelocityGradient,
    Viscosity, ViscousStress, newtonian_viscous_stress_kernel, strain_rate_tensor_kernel,
    viscous_dissipation_rate_kernel,
};

const TOL: f64 = 1e-12;

// =============================================================================
// Continuity
// =============================================================================

#[test]
fn test_continuity_zero_for_incompressible_divergence_free() {
    // Spec scenario: continuity ≡ 0 for incompressible divergence-free flow.
    let rho = Density::<f64>::new(1.225).unwrap();
    let u = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let grad_rho = [0.0_f64; 3]; // incompressible
    let div_u = 0.0_f64; // divergence-free
    let rhs = compressible_ns_continuity_rhs_kernel(&rho, &u, &grad_rho, div_u);
    assert!(rhs.abs() < TOL);
}

#[test]
fn test_continuity_known_value() {
    // ρ = 2, u = (1, 0, 0), ∇ρ = (3, 0, 0), ∇·u = 0.5
    // RHS = -(u·∇ρ + ρ·∇·u) = -(3 + 1) = -4
    let rho = Density::<f64>::new(2.0).unwrap();
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let grad_rho = [3.0_f64, 0.0, 0.0];
    let rhs = compressible_ns_continuity_rhs_kernel(&rho, &u, &grad_rho, 0.5);
    assert!((rhs - (-4.0)).abs() < TOL);
}

// =============================================================================
// Momentum
// =============================================================================

#[test]
fn test_momentum_reduces_to_incompressible_ns_at_constant_rho_divergence_free() {
    // Spec scenario: at constant ρ and ∇·u = 0 (so τ = 2μS, div τ = ρ ν ∇²u),
    // compressible momentum RHS matches incompressible NS.
    let u = Velocity3::<f64>::new([2.0, 1.0, 0.5]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[0.1, 0.0, 0.0], [0.0, -0.1, 0.0], [0.0, 0.0, 0.0]]).unwrap(); // divergence-free
    let lap = [0.5_f64, 0.7, 0.9];
    let gp = [1.0_f64, 2.0, 3.0];
    let rho = Density::<f64>::new(1.5).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.02).unwrap();
    let b = AccelerationVector::<f64>::new([0.0, 0.0, -9.81]).unwrap();

    // Build div_tau = ρ ν ∇²u for the constant-μ, divergence-free case.
    let r = 1.5_f64;
    let v = 0.02_f64;
    let div_tau = [r * v * lap[0], r * v * lap[1], r * v * lap[2]];

    let compr = compressible_ns_momentum_rhs_kernel(&u, &g, &gp, &div_tau, &rho, &b)
        .unwrap()
        .into_inner();
    let incompr = incompressible_ns_rhs_kernel(&u, &g, &lap, &gp, &rho, &nu, &b)
        .unwrap()
        .into_inner();
    for i in 0..3 {
        assert!((compr[i] - incompr[i]).abs() < TOL);
    }
}

#[test]
fn test_momentum_zero_density_errors() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g = VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap();
    let gp = [1.0_f64, 0.0, 0.0];
    let div_tau = [0.0_f64; 3];
    let rho = Density::<f64>::new(0.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0; 3]).unwrap();
    assert!(compressible_ns_momentum_rhs_kernel(&u, &g, &gp, &div_tau, &rho, &b).is_err());
}

// =============================================================================
// Energy
// =============================================================================

#[test]
fn test_energy_known_value() {
    // ρ = 1, u = (1, 0, 0), body = (g_x, 0, 0)
    // RHS = -div_rho_u_e - div_p_u + div_tau_dot_u - div_q + ρ·u·g
    //     = -2 - 3 + 4 - 5 + 1·1·6 = 0
    let rho = Density::<f64>::new(1.0).unwrap();
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let b = AccelerationVector::<f64>::new([6.0, 0.0, 0.0]).unwrap();
    let rhs = compressible_ns_energy_rhs_kernel(&rho, &u, 2.0, 3.0, 4.0, 5.0, &b);
    assert!(rhs.abs() < TOL);
}

#[test]
fn test_energy_dissipation_term_is_nonnegative_for_newtonian_fluid() {
    // Spec scenario: viscous dissipation Φ = τ:∇u ≥ 0 for any Newtonian fluid.
    // The energy equation's `div(τ·u)` decomposes as u·(div τ) + τ:∇u, and the
    // τ:∇u contribution is the dissipation. We verify the dissipation kernel
    // returns Φ ≥ 0 for an arbitrary smooth velocity gradient (the per-point
    // dissipation positivity carried through into the energy equation).
    let mu = Viscosity::<f64>::new(0.01).unwrap();
    // Multiple distinct test gradients (incompressible: trace = 0).
    let gradients = [
        [[0.5, 1.0, 0.0], [1.0, -0.3, 0.0], [0.0, 0.0, -0.2]],
        [[0.0, 0.5, 0.0], [-0.5, 0.0, 1.0], [0.0, -1.0, 0.0]],
        [[2.0, 0.3, -0.1], [0.3, -1.0, 0.5], [-0.1, 0.5, -1.0]],
    ];
    for gmat in gradients {
        let g = VelocityGradient::<f64>::new(gmat).unwrap();
        let s: StrainRateTensor<f64> = strain_rate_tensor_kernel(&g).unwrap();
        let div_u = gmat[0][0] + gmat[1][1] + gmat[2][2];
        let tau: ViscousStress<f64> = newtonian_viscous_stress_kernel(&mu, &s, div_u).unwrap();
        let phi = viscous_dissipation_rate_kernel(&tau, &g);
        assert!(phi >= 0.0, "Φ = {} should be ≥ 0", phi);
    }
}

#[test]
fn test_energy_linear_in_divergences() {
    // The energy kernel is affine in (div_rho_u_e, div_p_u, div_tau_dot_u, div_q).
    let rho = Density::<f64>::new(1.0).unwrap();
    let u = Velocity3::<f64>::new([0.0; 3]).unwrap();
    let b = AccelerationVector::<f64>::new([0.0; 3]).unwrap();
    // u·g = 0 so the body-force term vanishes; check superposition of the divs.
    let r1 = compressible_ns_energy_rhs_kernel(&rho, &u, 1.0, 0.0, 0.0, 0.0, &b);
    let r2 = compressible_ns_energy_rhs_kernel(&rho, &u, 0.0, 2.0, 0.0, 0.0, &b);
    let r3 = compressible_ns_energy_rhs_kernel(&rho, &u, 0.0, 0.0, 3.0, 0.0, &b);
    let r4 = compressible_ns_energy_rhs_kernel(&rho, &u, 0.0, 0.0, 0.0, 4.0, &b);
    let r_sum = compressible_ns_energy_rhs_kernel(&rho, &u, 1.0, 2.0, 3.0, 4.0, &b);
    assert!((r1 + r2 + r3 + r4 - r_sum).abs() < TOL);
}

// =============================================================================
// f32 precision sweep
// =============================================================================

#[test]
fn test_compressible_ns_f32_sweep() {
    let rho = Density::<f32>::new(1.0).unwrap();
    let u = Velocity3::<f32>::new([1.0, 0.0, 0.0]).unwrap();
    let grad_rho = [0.1_f32, 0.0, 0.0];
    let cont = compressible_ns_continuity_rhs_kernel(&rho, &u, &grad_rho, 0.0_f32);
    // -(1·0.1 + 1·0) = -0.1
    assert!((cont - (-0.1_f32)).abs() < 1e-5);

    let g = VelocityGradient::<f32>::new([[0.0; 3]; 3]).unwrap();
    let gp = [10.0_f32, 0.0, 0.0];
    let div_tau = [0.0_f32; 3];
    let b = AccelerationVector::<f32>::new([0.0, -9.81, 0.0]).unwrap();
    let m = compressible_ns_momentum_rhs_kernel(&u, &g, &gp, &div_tau, &rho, &b)
        .unwrap()
        .into_inner();
    // x: 0 -10 +0 + 0 = -10
    // y: 0 - 0 + 0 + -9.81 = -9.81
    assert!((m[0] - (-10.0_f32)).abs() < 1e-5);
    assert!((m[1] - (-9.81_f32)).abs() < 1e-4);
}
