/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Density, KinematicViscosity, PhysicsErrorEnum, Pressure, Velocity3, VelocityGradient,
    ViscousStress, VorticityVector, continuity_rhs_kernel, convective_acceleration_kernel,
    kinetic_energy_density_kernel, pressure_gradient_force_kernel, pressure_work_kernel,
    scalar_advection_diffusion_kernel, viscous_diffusion_kernel, viscous_dissipation_rate_kernel,
    vorticity_transport_kernel,
};

const TOL_F64: f64 = 1e-12;
const TOL_F32: f32 = 1e-5;

// =============================================================================
// convective_acceleration_kernel
// =============================================================================

#[test]
fn test_convective_acceleration_on_known_field() {
    // u = (1, 0, 0); grad_u[i][j] = 1 if (i,j) == (0,0) else 0  =>  (u·∇)u = (1, 0, 0)
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let a = convective_acceleration_kernel(&u, &g);
    assert!((a.value()[0] - 1.0).abs() < TOL_F64);
    assert!(a.value()[1].abs() < TOL_F64);
    assert!(a.value()[2].abs() < TOL_F64);
}

#[test]
fn test_convective_acceleration_zero_for_zero_velocity() {
    let u = Velocity3::<f64>::default();
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]).unwrap();
    let a = convective_acceleration_kernel(&u, &g);
    for c in a.value() {
        assert_eq!(*c, 0.0);
    }
}

#[test]
fn test_convective_acceleration_linearity_in_velocity_offset() {
    // (u + c)·∇u − u·∇u  should equal grad_u · c.
    let u_raw = [2.0, -1.0, 0.5];
    let c = [0.5, 1.0, -2.0];
    let u = Velocity3::<f64>::new(u_raw).unwrap();
    let u_shifted =
        Velocity3::<f64>::new([u_raw[0] + c[0], u_raw[1] + c[1], u_raw[2] + c[2]]).unwrap();
    let g = VelocityGradient::<f64>::new([[0.5, 1.0, -2.0], [3.0, 0.0, 0.5], [-1.5, 4.0, 2.0]])
        .unwrap();

    let a1 = convective_acceleration_kernel(&u, &g);
    let a2 = convective_acceleration_kernel(&u_shifted, &g);

    // Expected: (a2 - a1)_i = Σ_j grad_u[i][j] * c[j]
    let gv = g.value();
    let expected = [
        gv[0][0] * c[0] + gv[0][1] * c[1] + gv[0][2] * c[2],
        gv[1][0] * c[0] + gv[1][1] * c[1] + gv[1][2] * c[2],
        gv[2][0] * c[0] + gv[2][1] * c[1] + gv[2][2] * c[2],
    ];
    for ((a1_i, a2_i), exp_i) in a1
        .value()
        .iter()
        .zip(a2.value().iter())
        .zip(expected.iter())
    {
        let diff = a2_i - a1_i;
        assert!(
            (diff - exp_i).abs() < TOL_F64,
            "diff {} vs expected {}",
            diff,
            exp_i
        );
    }
}

#[test]
fn test_convective_acceleration_f32_precision() {
    let u = Velocity3::<f32>::new([1.0, 0.0, 0.0]).unwrap();
    let g =
        VelocityGradient::<f32>::new([[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let a = convective_acceleration_kernel(&u, &g);
    assert!((a.value()[0] - 1.0).abs() < TOL_F32);
}

// =============================================================================
// viscous_diffusion_kernel
// =============================================================================

#[test]
fn test_viscous_diffusion_on_known_input() {
    let nu = KinematicViscosity::<f64>::new(1.0e-3).unwrap();
    let lap_u = [10.0, -20.0, 5.0];
    let a = viscous_diffusion_kernel(&nu, &lap_u);
    assert!((a.value()[0] - 1.0e-2).abs() < TOL_F64);
    assert!((a.value()[1] + 2.0e-2).abs() < TOL_F64);
    assert!((a.value()[2] - 5.0e-3).abs() < TOL_F64);
}

#[test]
fn test_viscous_diffusion_zero_at_zero_viscosity() {
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    let lap_u = [1.0, 2.0, 3.0];
    let a = viscous_diffusion_kernel(&nu, &lap_u);
    for c in a.value() {
        assert_eq!(*c, 0.0);
    }
}

#[test]
fn test_viscous_diffusion_linear_in_viscosity() {
    let nu1 = KinematicViscosity::<f64>::new(1.0e-3).unwrap();
    let nu2 = KinematicViscosity::<f64>::new(2.0e-3).unwrap();
    let lap_u = [1.0, 1.0, 1.0];
    let a1 = viscous_diffusion_kernel(&nu1, &lap_u);
    let a2 = viscous_diffusion_kernel(&nu2, &lap_u);
    for i in 0..3 {
        assert!((a2.value()[i] - 2.0 * a1.value()[i]).abs() < TOL_F64);
    }
}

#[test]
fn test_viscous_diffusion_f32_precision() {
    let nu = KinematicViscosity::<f32>::new(1.0e-3).unwrap();
    let lap_u = [10.0_f32, 0.0, 0.0];
    let a = viscous_diffusion_kernel(&nu, &lap_u);
    assert!((a.value()[0] - 1.0e-2).abs() < TOL_F32);
}

// =============================================================================
// pressure_gradient_force_kernel
// =============================================================================

#[test]
fn test_pressure_gradient_force_on_known_input() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let grad_p = [10.0, 0.0, 0.0];
    let a = pressure_gradient_force_kernel(&rho, &grad_p).unwrap();
    // a = -(1/1000) * (10, 0, 0) = (-0.01, 0, 0)
    assert!((a.value()[0] + 0.01).abs() < TOL_F64);
    assert_eq!(a.value()[1], 0.0);
    assert_eq!(a.value()[2], 0.0);
}

#[test]
fn test_pressure_gradient_force_errors_on_zero_density() {
    let rho = Density::<f64>::new(0.0).unwrap();
    let grad_p = [1.0, 0.0, 0.0];
    let r = pressure_gradient_force_kernel(&rho, &grad_p);
    assert!(r.is_err());
    match &r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("density")),
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_pressure_gradient_force_zero_for_zero_gradient() {
    let rho = Density::<f64>::new(1.225).unwrap();
    let a = pressure_gradient_force_kernel(&rho, &[0.0; 3]).unwrap();
    for c in a.value() {
        assert_eq!(*c, 0.0);
    }
}

// =============================================================================
// continuity_rhs_kernel
// =============================================================================

#[test]
fn test_continuity_zero_for_incompressible_divergence_free() {
    let rho = Density::<f64>::new(1.0).unwrap();
    let u = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let rhs = continuity_rhs_kernel(&rho, &u, &[0.0; 3], 0.0);
    assert_eq!(rhs, 0.0);
}

#[test]
fn test_continuity_picks_up_density_advection() {
    // ρ constant => -(u·∇ρ) = 0. Test with ∇ρ ≠ 0 instead.
    let rho = Density::<f64>::new(1.0).unwrap();
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let grad_rho = [2.0, 0.0, 0.0];
    let rhs = continuity_rhs_kernel(&rho, &u, &grad_rho, 0.0);
    // expected: -(1*2 + 1*0) = -2
    assert!((rhs + 2.0).abs() < TOL_F64);
}

#[test]
fn test_continuity_picks_up_divergence_term() {
    let rho = Density::<f64>::new(3.0).unwrap();
    let u = Velocity3::<f64>::new([0.0, 0.0, 0.0]).unwrap();
    let rhs = continuity_rhs_kernel(&rho, &u, &[0.0; 3], 2.0);
    // expected: -(0 + 3*2) = -6
    assert!((rhs + 6.0).abs() < TOL_F64);
}

// =============================================================================
// vorticity_transport_kernel
// =============================================================================

#[test]
fn test_vorticity_transport_inviscid_helmholtz() {
    // At ν = 0, the kernel should equal -(u·∇)ω + (ω·∇)u.
    let omega = VorticityVector::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let u = Velocity3::<f64>::new([0.0, 1.0, 0.0]).unwrap();
    let grad_u =
        VelocityGradient::<f64>::new([[0.5, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let grad_omega = [[0.0; 3], [0.0, 0.1, 0.0], [0.0; 3]];
    let lap_omega = [0.0; 3];
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();

    let rhs = vorticity_transport_kernel(&omega, &u, &grad_u, &grad_omega, &lap_omega, &nu);

    // Expected:
    //   -(u·∇)ω: -(u_x * grad_omega[i][0] + u_y * grad_omega[i][1] + u_z * grad_omega[i][2])
    //     i=0: -(0 + 1·0.0 + 0) = 0
    //     i=1: -(0 + 1·0.1 + 0) = -0.1
    //     i=2: 0
    //   (ω·∇)u: ω_x * grad_u[i][0] + ω_y * grad_u[i][1] + ω_z * grad_u[i][2]
    //     i=0: 1·0.5 = 0.5
    //     i=1: 1·0 = 0
    //     i=2: 1·0 = 0
    //   sum: (0.5, -0.1, 0)
    assert!((rhs.value()[0] - 0.5).abs() < TOL_F64);
    assert!((rhs.value()[1] + 0.1).abs() < TOL_F64);
    assert!(rhs.value()[2].abs() < TOL_F64);
}

#[test]
fn test_vorticity_transport_diffusion_term_proportional_to_viscosity() {
    // Set u = 0 and ω so vortex stretching also vanishes; only the diffusion term remains.
    let omega = VorticityVector::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let u = Velocity3::<f64>::default();
    let grad_u = VelocityGradient::<f64>::default();
    let grad_omega = [[0.0; 3]; 3];
    let lap_omega = [4.0, 0.0, 0.0];
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();

    let rhs = vorticity_transport_kernel(&omega, &u, &grad_u, &grad_omega, &lap_omega, &nu);
    // Expected: ν * lap_omega = (0.5 * 4, 0, 0) = (2, 0, 0)
    assert!((rhs.value()[0] - 2.0).abs() < TOL_F64);
    assert!(rhs.value()[1].abs() < TOL_F64);
    assert!(rhs.value()[2].abs() < TOL_F64);
}

#[test]
fn test_vortex_stretching_vanishes_when_omega_orthogonal_to_grad_u_rows() {
    // ω points along z. grad_u rows are all along x — so ω·∇u rows are zero.
    let omega = VorticityVector::<f64>::new([0.0, 0.0, 1.0]).unwrap();
    let u = Velocity3::<f64>::default();
    let grad_u =
        VelocityGradient::<f64>::new([[1.0, 2.0, 0.0], [3.0, 4.0, 0.0], [5.0, 6.0, 0.0]]).unwrap();
    let grad_omega = [[0.0; 3]; 3];
    let lap_omega = [0.0; 3];
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();

    let rhs = vorticity_transport_kernel(&omega, &u, &grad_u, &grad_omega, &lap_omega, &nu);
    // (ω·∇)u_i = ω_z * grad_u[i][2] = 1 * 0 = 0 for every i. All zero.
    for c in rhs.value() {
        assert!(c.abs() < TOL_F64);
    }
}

// =============================================================================
// scalar_advection_diffusion_kernel
// =============================================================================

#[test]
fn test_scalar_pure_advection_at_zero_diffusivity() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let grad_phi = [2.0, 0.0, 0.0];
    let rhs = scalar_advection_diffusion_kernel(&u, &grad_phi, 100.0, 0.0, 0.0);
    // -u·∇φ = -2
    assert!((rhs + 2.0).abs() < TOL_F64);
}

#[test]
fn test_scalar_pure_diffusion_at_zero_velocity() {
    let u = Velocity3::<f64>::default();
    let grad_phi = [1.0, 2.0, 3.0];
    let rhs = scalar_advection_diffusion_kernel(&u, &grad_phi, 5.0, 0.1, 0.0);
    // D * ∇²φ = 0.1 * 5 = 0.5
    assert!((rhs - 0.5).abs() < TOL_F64);
}

#[test]
fn test_scalar_source_only_when_advection_and_diffusion_vanish() {
    let u = Velocity3::<f64>::default();
    let rhs = scalar_advection_diffusion_kernel(&u, &[0.0; 3], 0.0, 0.0, 7.5);
    assert_eq!(rhs, 7.5);
}

// =============================================================================
// Energy building blocks
// =============================================================================

#[test]
fn test_kinetic_energy_density_known_value() {
    let rho = Density::<f64>::new(2.0).unwrap();
    let u = Velocity3::<f64>::new([3.0, 4.0, 0.0]).unwrap();
    // 2 * 0.5 * (9 + 16) = 25
    let e = kinetic_energy_density_kernel(&rho, &u).unwrap();
    assert!((e - 25.0).abs() < TOL_F64);
}

#[test]
fn test_kinetic_energy_density_nonneg() {
    let rho = Density::<f64>::new(1.0).unwrap();
    let cases = [[0.0, 0.0, 0.0], [1.0, -2.0, 3.0], [-5.0, 0.0, 0.0]];
    for raw in cases {
        let u = Velocity3::<f64>::new(raw).unwrap();
        let e = kinetic_energy_density_kernel(&rho, &u).unwrap();
        assert!(e >= 0.0);
    }
}

#[test]
fn test_viscous_dissipation_double_contraction() {
    // τ:∇u = Σ τ_ij * grad_u[i][j].
    // Diagonal stress, diagonal gradient => sum of products of diagonals.
    let tau =
        ViscousStress::<f64>::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]).unwrap();
    let phi = viscous_dissipation_rate_kernel(&tau, &g);
    // 2 + 3 + 5 = 10
    assert!((phi - 10.0).abs() < TOL_F64);
}

#[test]
fn test_viscous_dissipation_zero_for_zero_stress() {
    let tau = ViscousStress::<f64>::default();
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]).unwrap();
    let phi = viscous_dissipation_rate_kernel(&tau, &g);
    assert_eq!(phi, 0.0);
}

#[test]
fn test_pressure_work_sign_agrees_with_div_u_at_positive_pressure() {
    let p = Pressure::<f64>::new(101325.0).unwrap();
    let w_positive = pressure_work_kernel(&p, 1.0);
    let w_negative = pressure_work_kernel(&p, -1.0);
    assert!(w_positive > 0.0);
    assert!(w_negative < 0.0);
    assert!((w_positive + w_negative).abs() < TOL_F64);
}

#[test]
fn test_pressure_work_zero_for_zero_divergence() {
    let p = Pressure::<f64>::new(101325.0).unwrap();
    let w = pressure_work_kernel(&p, 0.0);
    assert_eq!(w, 0.0);
}
