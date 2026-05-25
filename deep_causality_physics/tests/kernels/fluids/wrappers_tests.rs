/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    CauchyStress, Density, KinematicViscosity, Length, Pressure, Speed, StrainRateTensor,
    Velocity3, VelocityGradient, Viscosity, VorticityVector, bernoulli_pressure, bond_number,
    capillary_number, continuity_rhs, convective_acceleration, dissipation_rate, eckert_number,
    eddy_viscosity_boussinesq, enstrophy_density, froude_number, grashof_number, helicity_density,
    hydrostatic_pressure, integral_length_scale, kinetic_energy_density, knudsen_number,
    kolmogorov_length, kolmogorov_time, kolmogorov_velocity, lewis_number, mach_number,
    newtonian_viscous_stress, newtonian_viscous_stress_with_bulk, nusselt_number,
    particle_stokes_number, peclet_number, power_law_apparent_viscosity, prandtl_number,
    pressure_gradient_force, pressure_work, rayleigh_number, reynolds_number, reynolds_stress,
    richardson_number, rotation_rate_tensor, scalar_advection_diffusion, schmidt_number,
    strain_rate_tensor, strouhal_number, taylor_microscale, turbulent_kinetic_energy,
    velocity_gradient_invariants, viscous_diffusion, viscous_dissipation_rate,
    vorticity_from_gradient, vorticity_transport, weber_number,
};

// =============================================================================
// hydrostatic_pressure Wrapper Tests
// =============================================================================

#[test]
fn test_hydrostatic_pressure_wrapper_success() {
    let p0 = Pressure::new(101325.0).unwrap();
    let density = Density::new(1000.0).unwrap();
    let depth = Length::new(10.0).unwrap();

    let effect = hydrostatic_pressure(&p0, &density, &depth);
    assert!(effect.is_ok());

    let p = effect.value().clone().into_value().unwrap();
    assert!(p.value() > p0.value());
}

// =============================================================================
// bernoulli_pressure Wrapper Tests
// =============================================================================

#[test]
fn test_bernoulli_pressure_wrapper_success() {
    let p1 = Pressure::new(100000.0).unwrap();
    let v1 = Speed::new(5.0).unwrap();
    let h1 = Length::new(10.0).unwrap();
    let v2 = Speed::new(10.0).unwrap();
    let h2 = Length::new(5.0).unwrap();
    let density = Density::new(1000.0).unwrap();

    let effect = bernoulli_pressure(&p1, &v1, &h1, &v2, &h2, &density);
    assert!(effect.is_ok());
}

// =============================================================================
// Kinematic wrapper tests
// =============================================================================

#[test]
fn test_strain_rate_tensor_wrapper_success() {
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]).unwrap();
    let effect = strain_rate_tensor(&g);
    assert!(effect.is_ok());
    let s = effect.value().clone().into_value().unwrap();
    let raw = s.value();
    // Symmetric by construction
    assert!((raw[0][1] - raw[1][0]).abs() < 1e-12);
}

#[test]
fn test_rotation_rate_tensor_wrapper_success() {
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]).unwrap();
    let effect = rotation_rate_tensor(&g);
    assert!(effect.is_ok());
    let o = effect.value().clone().into_value().unwrap();
    // Antisymmetric: diagonal vanishes
    assert!(o.value()[0][0].abs() < 1e-12);
}

#[test]
fn test_vorticity_from_gradient_wrapper_success() {
    let g =
        VelocityGradient::<f64>::new([[0.0, 0.0, 0.0], [0.0, 0.0, -0.5], [0.0, 0.5, 0.0]]).unwrap();
    let effect = vorticity_from_gradient(&g);
    assert!(effect.is_ok());
    let w = effect.value().clone().into_value().unwrap();
    assert!((w.value()[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_velocity_gradient_invariants_wrapper_success() {
    let g =
        VelocityGradient::<f64>::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]).unwrap();
    let effect = velocity_gradient_invariants(&g);
    assert!(effect.is_ok());
    let (p, _q, _r) = effect.value().clone().into_value().unwrap();
    assert!((p - (-10.0)).abs() < 1e-12);
}

#[test]
fn test_helicity_density_wrapper_success() {
    let u = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let w = VorticityVector::<f64>::new([4.0, 5.0, 6.0]).unwrap();
    let effect = helicity_density(&u, &w);
    assert!(effect.is_ok());
    let h = effect.value().clone().into_value().unwrap();
    assert!((h - 32.0).abs() < 1e-12);
}

#[test]
fn test_enstrophy_density_wrapper_success() {
    let w = VorticityVector::<f64>::new([3.0, 4.0, 0.0]).unwrap();
    let effect = enstrophy_density(&w);
    assert!(effect.is_ok());
    let e = effect.value().clone().into_value().unwrap();
    assert!((e - 12.5).abs() < 1e-12);
}

// =============================================================================
// Governing kernel wrapper tests
// =============================================================================

#[test]
fn test_convective_acceleration_wrapper_success() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let effect = convective_acceleration(&u, &g);
    assert!(effect.is_ok());
    let a = effect.value().clone().into_value().unwrap();
    assert!((a.value()[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_viscous_diffusion_wrapper_success() {
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();
    let effect = viscous_diffusion(&nu, &[2.0, 0.0, 0.0]);
    assert!(effect.is_ok());
    let a = effect.value().clone().into_value().unwrap();
    assert!((a.value()[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_pressure_gradient_force_wrapper_success() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let effect = pressure_gradient_force(&rho, &[10.0, 0.0, 0.0]);
    assert!(effect.is_ok());
}

#[test]
fn test_pressure_gradient_force_wrapper_error_path() {
    let rho = Density::<f64>::new(0.0).unwrap();
    let effect = pressure_gradient_force(&rho, &[1.0, 0.0, 0.0]);
    assert!(!effect.is_ok());
}

#[test]
fn test_continuity_rhs_wrapper_success() {
    let rho = Density::<f64>::new(1.0).unwrap();
    let u = Velocity3::<f64>::new([0.0; 3]).unwrap();
    let effect = continuity_rhs(&rho, &u, &[0.0; 3], 0.0);
    assert!(effect.is_ok());
    assert_eq!(effect.value().clone().into_value().unwrap(), 0.0);
}

#[test]
fn test_vorticity_transport_wrapper_success() {
    let omega = VorticityVector::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let u = Velocity3::<f64>::default();
    let grad_u = VelocityGradient::<f64>::default();
    let grad_omega = [[0.0; 3]; 3];
    let lap_omega = [4.0, 0.0, 0.0];
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();
    let effect = vorticity_transport(&omega, &u, &grad_u, &grad_omega, &lap_omega, &nu);
    assert!(effect.is_ok());
    let a = effect.value().clone().into_value().unwrap();
    assert!((a.value()[0] - 2.0).abs() < 1e-12);
}

#[test]
fn test_scalar_advection_diffusion_wrapper_success() {
    let u = Velocity3::<f64>::default();
    let effect = scalar_advection_diffusion(&u, &[0.0; 3], 0.0, 0.0, 7.5);
    assert!(effect.is_ok());
    assert_eq!(effect.value().clone().into_value().unwrap(), 7.5);
}

#[test]
fn test_kinetic_energy_density_wrapper_success() {
    let rho = Density::<f64>::new(2.0).unwrap();
    let u = Velocity3::<f64>::new([3.0, 4.0, 0.0]).unwrap();
    let effect = kinetic_energy_density(&rho, &u);
    assert!(effect.is_ok());
    let e = effect.value().clone().into_value().unwrap();
    assert!((e - 25.0).abs() < 1e-12);
}

#[test]
fn test_viscous_dissipation_rate_wrapper_success() {
    let tau =
        CauchyStress::<f64>::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]).unwrap();
    let effect = viscous_dissipation_rate(&tau, &g);
    assert!(effect.is_ok());
    let phi = effect.value().clone().into_value().unwrap();
    assert!((phi - 10.0).abs() < 1e-12);
}

#[test]
fn test_pressure_work_wrapper_success() {
    let p = Pressure::<f64>::new(2.0).unwrap();
    let effect = pressure_work(&p, 3.0);
    assert!(effect.is_ok());
    assert_eq!(effect.value().clone().into_value().unwrap(), 6.0);
}

// =============================================================================
// Constitutive kernel wrapper tests
// =============================================================================

#[test]
fn test_newtonian_viscous_stress_wrapper_success() {
    let mu = Viscosity::<f64>::new(0.5).unwrap();
    let s =
        StrainRateTensor::<f64>::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]).unwrap();
    let effect = newtonian_viscous_stress(&mu, &s, 0.0);
    assert!(effect.is_ok());
    let tau = effect.value().clone().into_value().unwrap();
    // τ_00 = 2 * 0.5 * 1 = 1
    assert!((tau.value()[0][0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_newtonian_viscous_stress_with_bulk_wrapper_success() {
    let mu = Viscosity::<f64>::new(0.0).unwrap();
    let zeta = Viscosity::<f64>::new(1.0).unwrap();
    let s = StrainRateTensor::<f64>::default();
    let effect = newtonian_viscous_stress_with_bulk(&mu, &zeta, &s, 5.0);
    assert!(effect.is_ok());
    let tau = effect.value().clone().into_value().unwrap();
    // Diagonal: (-0 + 1) * 5 = 5
    assert!((tau.value()[0][0] - 5.0).abs() < 1e-12);
}

#[test]
fn test_power_law_apparent_viscosity_wrapper_success() {
    let effect = power_law_apparent_viscosity(2.0_f64, 0.5, 4.0);
    assert!(effect.is_ok());
    let mu_eff = effect.value().clone().into_value().unwrap();
    assert!((mu_eff.value() - 1.0).abs() < 1e-12);
}

#[test]
fn test_power_law_apparent_viscosity_wrapper_error_path() {
    let effect = power_law_apparent_viscosity(1.0_f64, 0.5, -0.1);
    assert!(!effect.is_ok());
}

// =============================================================================
// Dimensionless number wrapper smoke tests
// =============================================================================

#[test]
fn test_reynolds_number_wrapper() {
    let u = Speed::<f64>::new(2.0).unwrap();
    let l = Length::<f64>::new(0.1).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.0e-3).unwrap();
    let effect = reynolds_number(&u, &l, &nu);
    assert!(effect.is_ok());
    assert!((effect.value().clone().into_value().unwrap() - 200.0).abs() < 1e-10);
}

#[test]
fn test_reynolds_number_wrapper_error_path() {
    let u = Speed::<f64>::new(1.0).unwrap();
    let l = Length::<f64>::new(1.0).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    let effect = reynolds_number(&u, &l, &nu);
    assert!(!effect.is_ok());
}

#[test]
fn test_mach_number_wrapper() {
    let u = Speed::<f64>::new(170.0).unwrap();
    let a = Speed::<f64>::new(340.0).unwrap();
    assert!(mach_number(&u, &a).is_ok());
}

#[test]
fn test_froude_number_wrapper() {
    let u = Speed::<f64>::new(10.0).unwrap();
    let l = Length::<f64>::new(2.5).unwrap();
    assert!(froude_number(&u, 9.8_f64, &l).is_ok());
}

#[test]
fn test_weber_number_wrapper() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let u = Speed::<f64>::new(2.0).unwrap();
    let l = Length::<f64>::new(0.001).unwrap();
    assert!(weber_number(&rho, &u, &l, 0.072_f64).is_ok());
}

#[test]
fn test_prandtl_number_wrapper() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(prandtl_number(&nu, 2.1e-5_f64).is_ok());
}

#[test]
fn test_peclet_number_wrapper() {
    let u = Speed::<f64>::new(2.0).unwrap();
    let l = Length::<f64>::new(0.1).unwrap();
    assert!(peclet_number(&u, &l, 2.0e-5_f64).is_ok());
}

#[test]
fn test_strouhal_number_wrapper() {
    let u = Speed::<f64>::new(5.0).unwrap();
    let l = Length::<f64>::new(0.1).unwrap();
    assert!(strouhal_number(10.0_f64, &l, &u).is_ok());
}

#[test]
fn test_knudsen_number_wrapper() {
    let l = Length::<f64>::new(1.0e-6).unwrap();
    assert!(knudsen_number(1.0e-7_f64, &l).is_ok());
}

#[test]
fn test_richardson_number_wrapper() {
    let u = Speed::<f64>::new(2.0).unwrap();
    let l = Length::<f64>::new(1.0).unwrap();
    assert!(richardson_number(9.8_f64, 3.0e-3, 10.0, &l, &u).is_ok());
}

#[test]
fn test_rayleigh_number_wrapper() {
    let l = Length::<f64>::new(0.1).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(rayleigh_number(9.8_f64, 3.0e-3, 10.0, &l, &nu, 2.1e-5).is_ok());
}

#[test]
fn test_grashof_number_wrapper() {
    let l = Length::<f64>::new(0.1).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(grashof_number(9.8_f64, 3.0e-3, 10.0, &l, &nu).is_ok());
}

#[test]
fn test_eckert_number_wrapper() {
    let u = Speed::<f64>::new(10.0).unwrap();
    assert!(eckert_number(&u, 1000.0_f64, 5.0).is_ok());
}

#[test]
fn test_schmidt_number_wrapper() {
    let nu = KinematicViscosity::<f64>::new(1.0e-6).unwrap();
    assert!(schmidt_number(&nu, 2.0e-9_f64).is_ok());
}

#[test]
fn test_lewis_number_wrapper() {
    assert!(lewis_number(2.0e-5_f64, 5.0e-9_f64).is_ok());
}

#[test]
fn test_particle_stokes_number_wrapper() {
    let u = Speed::<f64>::new(10.0).unwrap();
    let l = Length::<f64>::new(0.01).unwrap();
    assert!(particle_stokes_number(1.0e-3_f64, &u, &l).is_ok());
}

#[test]
fn test_capillary_number_wrapper() {
    let mu = Viscosity::<f64>::new(0.001).unwrap();
    let u = Speed::<f64>::new(1.0).unwrap();
    assert!(capillary_number(&mu, &u, 0.072_f64).is_ok());
}

#[test]
fn test_bond_number_wrapper() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let l = Length::<f64>::new(0.01).unwrap();
    assert!(bond_number(&rho, 9.8_f64, &l, 0.072).is_ok());
}

#[test]
fn test_nusselt_number_wrapper() {
    let l = Length::<f64>::new(0.1).unwrap();
    assert!(nusselt_number(100.0_f64, &l, 0.5).is_ok());
}

// =============================================================================
// Turbulence wrapper tests
// =============================================================================

#[test]
fn test_turbulent_kinetic_energy_wrapper() {
    let u = Velocity3::<f64>::new([3.0, 4.0, 0.0]).unwrap();
    let effect = turbulent_kinetic_energy(&u);
    assert!(effect.is_ok());
    assert!((effect.value().clone().into_value().unwrap() - 12.5).abs() < 1e-12);
}

#[test]
fn test_dissipation_rate_wrapper() {
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    assert!(dissipation_rate(&nu, &g).is_ok());
}

#[test]
fn test_kolmogorov_length_wrapper() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(kolmogorov_length(&nu, 1.0e-3_f64).is_ok());
}

#[test]
fn test_kolmogorov_length_wrapper_error_path() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(!kolmogorov_length(&nu, 0.0_f64).is_ok());
}

#[test]
fn test_kolmogorov_time_wrapper() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(kolmogorov_time(&nu, 1.0e-3_f64).is_ok());
}

#[test]
fn test_kolmogorov_velocity_wrapper() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(kolmogorov_velocity(&nu, 1.0e-3_f64).is_ok());
}

#[test]
fn test_taylor_microscale_wrapper() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(taylor_microscale(2.0_f64, 1.0e-2, &nu).is_ok());
}

#[test]
fn test_integral_length_scale_wrapper() {
    assert!(integral_length_scale(4.0_f64, 8.0).is_ok());
}

#[test]
fn test_reynolds_stress_wrapper() {
    let r_in = StrainRateTensor::<f64>::new([[1.0, 0.5, 0.2], [0.5, 2.0, -0.1], [0.2, -0.1, 0.8]])
        .unwrap();
    assert!(reynolds_stress(&r_in).is_ok());
}

#[test]
fn test_eddy_viscosity_boussinesq_wrapper_success() {
    let gamma = 2.0_f64;
    let k = 1.0_f64;
    let nu_t_target = 0.05;
    let r_xy = -(2.0 * nu_t_target) * (0.5 * gamma);
    let r = CauchyStress::<f64>::new([
        [(2.0 / 3.0) * k, r_xy, 0.0],
        [r_xy, (2.0 / 3.0) * k, 0.0],
        [0.0, 0.0, (2.0 / 3.0) * k],
    ])
    .unwrap();
    let s = StrainRateTensor::<f64>::new([
        [0.0, 0.5 * gamma, 0.0],
        [0.5 * gamma, 0.0, 0.0],
        [0.0, 0.0, 0.0],
    ])
    .unwrap();
    let effect = eddy_viscosity_boussinesq(&r, &s, k);
    assert!(effect.is_ok());
}

#[test]
fn test_eddy_viscosity_boussinesq_wrapper_error_path() {
    let r = CauchyStress::<f64>::default();
    let s = StrainRateTensor::<f64>::default();
    assert!(!eddy_viscosity_boussinesq(&r, &s, 0.5).is_ok());
}
