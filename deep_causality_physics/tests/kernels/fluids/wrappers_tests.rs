/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Density, KinematicViscosity, Length, Pressure, ReynoldsStress, SpecificEnthalpy, Speed,
    StrainRateTensor, Temperature, Velocity3, VelocityGradient, Viscosity, ViscousStress,
    VorticityVector, WallShearStress, area_mach_ratio, area_mach_ratio_kernel, bernoulli_pressure,
    bernoulli_total_head, bond_number, capillary_number, circulation, continuity_rhs,
    convective_acceleration, delta_criterion, dissipation_rate, dynamic_pressure,
    dynamic_pressure_kernel, eckert_number, eddy_viscosity_boussinesq,
    eddy_viscosity_boussinesq_kernel, enstrophy_density, entropy_production_rate,
    friction_velocity, froude_number, grashof_number, helicity_density, hydrostatic_pressure,
    integral_length_scale, isentropic_density_ratio, isentropic_density_ratio_kernel,
    isentropic_pressure_ratio, isentropic_pressure_ratio_kernel, isentropic_temperature_ratio,
    isentropic_temperature_ratio_kernel, kinetic_energy_density, knudsen_number, kolmogorov_length,
    kolmogorov_time, kolmogorov_velocity, kutta_joukowski_lift, lambda2, lambda2_kernel,
    lewis_number, log_law_velocity, mach_number, newtonian_viscous_stress,
    newtonian_viscous_stress_with_bulk, nusselt_number, particle_stokes_number, peclet_number,
    power_law_apparent_viscosity, prandtl_number, pressure_gradient_force, pressure_work,
    q_criterion, q_criterion_kernel, rayleigh_number, reynolds_number, reynolds_number_kernel,
    reynolds_stress, richardson_number, rotation_rate_tensor, scalar_advection_diffusion,
    schmidt_number, skin_friction_coefficient, skin_friction_coefficient_kernel, specific_enthalpy,
    speed_of_sound_ideal_gas, strain_rate_tensor, stream_function_2d, strouhal_number,
    swirling_strength, swirling_strength_kernel, taylor_microscale, total_enthalpy,
    total_pressure_isentropic, total_temperature_isentropic, total_temperature_isentropic_kernel,
    turbulent_kinetic_energy, turbulent_kinetic_energy_kernel, velocity_gradient_invariants,
    velocity_potential_2d, velocity_potential_2d_kernel, viscous_diffusion,
    viscous_dissipation_rate, viscous_length_scale, viscous_sublayer_velocity,
    viscous_sublayer_velocity_kernel, vorticity_from_gradient, vorticity_transport,
    wall_shear_stress_newtonian, weber_number, y_plus,
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

    let p = effect.value_cloned().unwrap();
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
    let s = effect.value_cloned().unwrap();
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
    let o = effect.value_cloned().unwrap();
    // Antisymmetric: diagonal vanishes
    assert!(o.value()[0][0].abs() < 1e-12);
}

#[test]
fn test_vorticity_from_gradient_wrapper_success() {
    let g =
        VelocityGradient::<f64>::new([[0.0, 0.0, 0.0], [0.0, 0.0, -0.5], [0.0, 0.5, 0.0]]).unwrap();
    let effect = vorticity_from_gradient(&g);
    assert!(effect.is_ok());
    let w = effect.value_cloned().unwrap();
    assert!((w.value()[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_velocity_gradient_invariants_wrapper_success() {
    let g =
        VelocityGradient::<f64>::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]).unwrap();
    let effect = velocity_gradient_invariants(&g);
    assert!(effect.is_ok());
    let (p, _q, _r) = effect.value_cloned().unwrap();
    assert!((p - (-10.0)).abs() < 1e-12);
}

#[test]
fn test_helicity_density_wrapper_success() {
    let u = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let w = VorticityVector::<f64>::new([4.0, 5.0, 6.0]).unwrap();
    let effect = helicity_density(&u, &w);
    assert!(effect.is_ok());
    let h = effect.value_cloned().unwrap();
    assert!((h - 32.0).abs() < 1e-12);
}

#[test]
fn test_enstrophy_density_wrapper_success() {
    let w = VorticityVector::<f64>::new([3.0, 4.0, 0.0]).unwrap();
    let effect = enstrophy_density(&w);
    assert!(effect.is_ok());
    let e = effect.value_cloned().unwrap();
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
    let a = effect.value_cloned().unwrap();
    assert!((a.value()[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_viscous_diffusion_wrapper_success() {
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();
    let effect = viscous_diffusion(&nu, &[2.0, 0.0, 0.0]);
    assert!(effect.is_ok());
    let a = effect.value_cloned().unwrap();
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
    assert_eq!(effect.value_cloned().unwrap(), 0.0);
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
    let a = effect.value_cloned().unwrap();
    assert!((a.value()[0] - 2.0).abs() < 1e-12);
}

#[test]
fn test_scalar_advection_diffusion_wrapper_success() {
    let u = Velocity3::<f64>::default();
    let effect = scalar_advection_diffusion(&u, &[0.0; 3], 0.0, 0.0, 7.5);
    assert!(effect.is_ok());
    assert_eq!(effect.value_cloned().unwrap(), 7.5);
}

#[test]
fn test_kinetic_energy_density_wrapper_success() {
    let rho = Density::<f64>::new(2.0).unwrap();
    let u = Velocity3::<f64>::new([3.0, 4.0, 0.0]).unwrap();
    let effect = kinetic_energy_density(&rho, &u);
    assert!(effect.is_ok());
    let e = effect.value_cloned().unwrap();
    assert!((e - 25.0).abs() < 1e-12);
}

#[test]
fn test_viscous_dissipation_rate_wrapper_success() {
    let tau =
        ViscousStress::<f64>::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]).unwrap();
    let effect = viscous_dissipation_rate(&tau, &g);
    assert!(effect.is_ok());
    let phi = effect.value_cloned().unwrap();
    assert!((phi - 10.0).abs() < 1e-12);
}

#[test]
fn test_pressure_work_wrapper_success() {
    let p = Pressure::<f64>::new(2.0).unwrap();
    let effect = pressure_work(&p, 3.0);
    assert!(effect.is_ok());
    assert_eq!(effect.value_cloned().unwrap(), 6.0);
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
    let tau = effect.value_cloned().unwrap();
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
    let tau = effect.value_cloned().unwrap();
    // Diagonal: (-0 + 1) * 5 = 5
    assert!((tau.value()[0][0] - 5.0).abs() < 1e-12);
}

#[test]
fn test_power_law_apparent_viscosity_wrapper_success() {
    let effect = power_law_apparent_viscosity(2.0_f64, 0.5, 4.0);
    assert!(effect.is_ok());
    let mu_eff = effect.value_cloned().unwrap();
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
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        reynolds_number_kernel(&u, &l, &nu).unwrap(),
        "reynolds_number must carry the value its kernel produced"
    );
    assert!((effect.value_cloned().unwrap() - 200.0).abs() < 1e-10);
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
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        turbulent_kinetic_energy_kernel(&u).unwrap(),
        "turbulent_kinetic_energy must carry the value its kernel produced"
    );
    assert!((effect.value_cloned().unwrap() - 12.5).abs() < 1e-12);
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
fn test_kolmogorov_time_wrapper_error_path() {
    // epsilon ≤ 0 ⇒ require_positive rejects.
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(!kolmogorov_time(&nu, 0.0_f64).is_ok());
}

#[test]
fn test_kolmogorov_velocity_wrapper() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(kolmogorov_velocity(&nu, 1.0e-3_f64).is_ok());
}

#[test]
fn test_kolmogorov_velocity_wrapper_error_path() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(!kolmogorov_velocity(&nu, 0.0_f64).is_ok());
}

#[test]
fn test_taylor_microscale_wrapper() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(taylor_microscale(2.0_f64, 1.0e-2, &nu).is_ok());
}

#[test]
fn test_taylor_microscale_wrapper_error_path() {
    // epsilon ≤ 0 ⇒ require_positive rejects.
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(!taylor_microscale(2.0_f64, 0.0, &nu).is_ok());
}

#[test]
fn test_integral_length_scale_wrapper() {
    assert!(integral_length_scale(4.0_f64, 8.0).is_ok());
}

#[test]
fn test_integral_length_scale_wrapper_error_path() {
    // epsilon ≤ 0 ⇒ require_positive rejects.
    assert!(!integral_length_scale(4.0_f64, 0.0).is_ok());
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
    let r = ReynoldsStress::<f64>::new([
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
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        eddy_viscosity_boussinesq_kernel(&r, &s, k).unwrap(),
        "eddy_viscosity_boussinesq must carry the value its kernel produced"
    );
}

#[test]
fn test_eddy_viscosity_boussinesq_wrapper_error_path() {
    let r = ReynoldsStress::<f64>::default();
    let s = StrainRateTensor::<f64>::default();
    assert!(!eddy_viscosity_boussinesq(&r, &s, 0.5).is_ok());
}

// =============================================================================
// Coherent-structure detector wrappers
// =============================================================================

#[test]
fn test_q_criterion_wrapper() {
    let g =
        VelocityGradient::<f64>::new([[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let effect = q_criterion(&g);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        q_criterion_kernel(&g).unwrap(),
        "q_criterion must carry the value its kernel produced"
    );
    assert!((effect.value_cloned().unwrap() - 1.0).abs() < 1e-12);
}

#[test]
fn test_delta_criterion_wrapper() {
    let g =
        VelocityGradient::<f64>::new([[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    assert!(delta_criterion(&g).is_ok());
}

#[test]
fn test_lambda2_wrapper() {
    let g =
        VelocityGradient::<f64>::new([[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let effect = lambda2(&g);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        lambda2_kernel(&g).unwrap(),
        "lambda2 must carry the value its kernel produced"
    );
    let v = effect.value_cloned().unwrap();
    assert!(v < 0.0);
}

#[test]
fn test_swirling_strength_wrapper() {
    let g =
        VelocityGradient::<f64>::new([[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let effect = swirling_strength(&g);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        swirling_strength_kernel(&g).unwrap(),
        "swirling_strength must carry the value its kernel produced"
    );
    let v = effect.value_cloned().unwrap();
    assert!((v - 1.0).abs() < 1e-12);
}

// =============================================================================
// Compressible-flow wrapper tests
// =============================================================================

#[test]
fn test_speed_of_sound_ideal_gas_wrapper() {
    let t = Temperature::<f64>::new(293.15).unwrap();
    assert!(speed_of_sound_ideal_gas(1.4_f64, 287.05, &t).is_ok());
}

#[test]
fn test_speed_of_sound_ideal_gas_wrapper_error_path() {
    let t = Temperature::<f64>::new(0.0).unwrap();
    assert!(!speed_of_sound_ideal_gas(1.4_f64, 287.05, &t).is_ok());
}

#[test]
fn test_specific_enthalpy_wrapper() {
    let t = Temperature::<f64>::new(300.0).unwrap();
    assert!(specific_enthalpy(1005.0_f64, &t).is_ok());
}

#[test]
fn test_total_enthalpy_wrapper() {
    let h = SpecificEnthalpy::<f64>::new(3.0e5).unwrap();
    let u = Velocity3::<f64>::new([100.0, 0.0, 0.0]).unwrap();
    assert!(total_enthalpy(&h, &u).is_ok());
}

#[test]
fn test_total_pressure_isentropic_wrapper() {
    let p = Pressure::<f64>::new(101_325.0).unwrap();
    assert!(total_pressure_isentropic(&p, 0.5_f64, 1.4).is_ok());
}

#[test]
fn test_total_temperature_isentropic_wrapper() {
    let t = Temperature::<f64>::new(300.0).unwrap();
    let effect = total_temperature_isentropic(&t, 1.0_f64, 1.4);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        total_temperature_isentropic_kernel(&t, 1.0_f64, 1.4).unwrap(),
        "total_temperature_isentropic must carry the value its kernel produced"
    );
    let v = effect.value_cloned().unwrap();
    assert!((v.value() - 360.0).abs() < 1e-6);
}

#[test]
fn test_entropy_production_rate_wrapper() {
    let tau = ViscousStress::<f64>::default();
    let grad_u = VelocityGradient::<f64>::default();
    let t = Temperature::<f64>::new(300.0).unwrap();
    assert!(entropy_production_rate(&t, &tau, &grad_u, 0.025_f64, &[10.0, 0.0, 0.0]).is_ok());
}

#[test]
fn test_entropy_production_rate_wrapper_error_path() {
    let tau = ViscousStress::<f64>::default();
    let grad_u = VelocityGradient::<f64>::default();
    let t = Temperature::<f64>::new(0.0).unwrap();
    assert!(!entropy_production_rate(&t, &tau, &grad_u, 1.0_f64, &[0.0; 3]).is_ok());
}

// =============================================================================
// Boundary-layer wrapper tests
// =============================================================================

#[test]
fn test_wall_shear_stress_newtonian_wrapper() {
    let mu = Viscosity::<f64>::new(1.0e-3).unwrap();
    let effect = wall_shear_stress_newtonian(&mu, 100.0_f64);
    assert!(effect.is_ok());
    let v = effect.value_cloned().unwrap();
    assert!((v.value() - 0.1).abs() < 1e-12);
}

#[test]
fn test_friction_velocity_wrapper() {
    let tau = WallShearStress::<f64>::new(0.1).unwrap();
    let rho = Density::<f64>::new(1.0).unwrap();
    assert!(friction_velocity(&tau, &rho).is_ok());
}

#[test]
fn test_friction_velocity_wrapper_error_path() {
    let tau = WallShearStress::<f64>::new(1.0).unwrap();
    let rho = Density::<f64>::new(0.0).unwrap();
    assert!(!friction_velocity(&tau, &rho).is_ok());
}

#[test]
fn test_viscous_length_scale_wrapper() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    let u_tau = Speed::<f64>::new(0.5).unwrap();
    assert!(viscous_length_scale(&nu, &u_tau).is_ok());
}

#[test]
fn test_viscous_length_scale_wrapper_error_path() {
    // u_tau = 0 ⇒ friction velocity is zero.
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    let u_tau = Speed::<f64>::new(0.0).unwrap();
    assert!(!viscous_length_scale(&nu, &u_tau).is_ok());
}

#[test]
fn test_y_plus_wrapper() {
    let y = Length::<f64>::new(1.0e-4).unwrap();
    let u_tau = Speed::<f64>::new(0.5).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    assert!(y_plus(&y, &u_tau, &nu).is_ok());
}

#[test]
fn test_y_plus_wrapper_error_path() {
    // nu = 0 ⇒ kinematic viscosity is zero.
    let y = Length::<f64>::new(1.0e-4).unwrap();
    let u_tau = Speed::<f64>::new(0.5).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    assert!(!y_plus(&y, &u_tau, &nu).is_ok());
}

#[test]
fn test_viscous_sublayer_velocity_wrapper() {
    let effect = viscous_sublayer_velocity(3.0_f64);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        viscous_sublayer_velocity_kernel(3.0_f64),
        "viscous_sublayer_velocity must carry the value its kernel produced"
    );
    assert_eq!(effect.value_cloned().unwrap(), 3.0);
}

#[test]
fn test_log_law_velocity_wrapper() {
    assert!(log_law_velocity(100.0_f64, 0.41, 5.0).is_ok());
}

#[test]
fn test_log_law_velocity_wrapper_error_path() {
    assert!(!log_law_velocity(0.0_f64, 0.41, 5.0).is_ok());
}

#[test]
fn test_skin_friction_coefficient_wrapper() {
    let tau = WallShearStress::<f64>::new(0.5).unwrap();
    let rho = Density::<f64>::new(1.0).unwrap();
    let u_inf = Speed::<f64>::new(10.0).unwrap();
    let effect = skin_friction_coefficient(&tau, &rho, &u_inf);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        skin_friction_coefficient_kernel(&tau, &rho, &u_inf).unwrap(),
        "skin_friction_coefficient must carry the value its kernel produced"
    );
    assert!((effect.value_cloned().unwrap() - 0.01).abs() < 1e-12);
}

#[test]
fn test_skin_friction_coefficient_wrapper_error_path() {
    // u_inf = 0 ⇒ division by zero guard.
    let tau = WallShearStress::<f64>::new(0.5).unwrap();
    let rho = Density::<f64>::new(1.0).unwrap();
    let u_inf = Speed::<f64>::new(0.0).unwrap();
    assert!(!skin_friction_coefficient(&tau, &rho, &u_inf).is_ok());
}

// =============================================================================
// Ideal-flow wrapper tests
// =============================================================================

#[test]
fn test_dynamic_pressure_wrapper() {
    let rho = Density::<f64>::new(1.225).unwrap();
    let u = Speed::<f64>::new(20.0).unwrap();
    let effect = dynamic_pressure(&rho, &u);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        dynamic_pressure_kernel(&rho, &u).unwrap(),
        "dynamic_pressure must carry the value its kernel produced"
    );
    assert!((effect.value_cloned().unwrap().value() - 245.0).abs() < 1e-12);
}

#[test]
fn test_bernoulli_total_head_wrapper() {
    let p = Pressure::<f64>::new(0.0).unwrap();
    let rho = Density::<f64>::new(1000.0).unwrap();
    let u = Speed::<f64>::new(0.0).unwrap();
    let h = Length::<f64>::new(5.0).unwrap();
    assert!(bernoulli_total_head(&p, &rho, &u, &h).is_ok());
}

#[test]
fn test_bernoulli_total_head_wrapper_error_path() {
    let p = Pressure::<f64>::new(101_325.0).unwrap();
    let rho = Density::<f64>::new(0.0).unwrap();
    let u = Speed::<f64>::new(1.0).unwrap();
    let h = Length::<f64>::new(0.0).unwrap();
    assert!(!bernoulli_total_head(&p, &rho, &u, &h).is_ok());
}

#[test]
fn test_stream_function_2d_wrapper() {
    let effect = stream_function_2d(1.0_f64, 0.0, 0.5, 0.3);
    assert!(effect.is_ok());
    assert_eq!(effect.value_cloned().unwrap(), 0.3);
}

#[test]
fn test_velocity_potential_2d_wrapper() {
    let effect = velocity_potential_2d(2.0_f64, 3.0, 1.0, 1.0);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        velocity_potential_2d_kernel(2.0_f64, 3.0, 1.0, 1.0),
        "velocity_potential_2d must carry the value its kernel produced"
    );
    assert_eq!(effect.value_cloned().unwrap(), 5.0);
}

#[test]
fn test_circulation_wrapper() {
    let velocities = vec![Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap(); 2];
    let tangents: Vec<[f64; 3]> = vec![[1.0, 0.0, 0.0]; 2];
    assert!(circulation(&velocities, &tangents).is_ok());
}

#[test]
fn test_circulation_wrapper_error_path() {
    let velocities = vec![Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap(); 3];
    let tangents: Vec<[f64; 3]> = vec![[1.0, 0.0, 0.0]; 2];
    assert!(!circulation(&velocities, &tangents).is_ok());
}

#[test]
fn test_kutta_joukowski_lift_wrapper() {
    let rho = Density::<f64>::new(1.225).unwrap();
    let u_inf = Speed::<f64>::new(50.0).unwrap();
    let effect = kutta_joukowski_lift(&rho, &u_inf, 10.0_f64);
    assert!(effect.is_ok());
    assert!((effect.value_cloned().unwrap() - 612.5).abs() < 1e-12);
}

// =============================================================================
// Additional wrapper error-path coverage
// =============================================================================

#[test]
fn test_bernoulli_pressure_wrapper_error_path() {
    // Large velocity-2 with tiny p1 ⇒ resulting pressure goes negative,
    // which Pressure::new rejects.
    let p1 = Pressure::<f64>::new(0.0).unwrap();
    let v1 = Speed::<f64>::new(0.0).unwrap();
    let h1 = Length::<f64>::new(0.0).unwrap();
    let v2 = Speed::<f64>::new(100.0).unwrap();
    let h2 = Length::<f64>::new(0.0).unwrap();
    let rho = Density::<f64>::new(1000.0).unwrap();
    let effect = bernoulli_pressure(&p1, &v1, &h1, &v2, &h2, &rho);
    assert!(!effect.is_ok());
}

#[test]
fn test_mach_number_wrapper_error_path() {
    let u = Speed::<f64>::new(100.0).unwrap();
    let c = Speed::<f64>::new(0.0).unwrap();
    let effect = mach_number(&u, &c);
    assert!(!effect.is_ok());
}

#[test]
fn test_froude_number_wrapper_error_path() {
    // gravity · length = 0 should error (would divide by zero in sqrt).
    let u = Speed::<f64>::new(1.0).unwrap();
    let length = Length::<f64>::new(1.0).unwrap();
    let effect = froude_number(&u, 0.0_f64, &length);
    assert!(!effect.is_ok());
}

#[test]
fn test_weber_number_wrapper_error_path() {
    // surface_tension = 0 ⇒ division by zero.
    let rho = Density::<f64>::new(1000.0).unwrap();
    let u = Speed::<f64>::new(1.0).unwrap();
    let length = Length::<f64>::new(0.01).unwrap();
    let effect = weber_number(&rho, &u, &length, 0.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_prandtl_number_wrapper_error_path() {
    // thermal_diffusivity = 0 ⇒ division by zero.
    let nu = KinematicViscosity::<f64>::new(1.0e-6).unwrap();
    let effect = prandtl_number(&nu, 0.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_peclet_number_wrapper_error_path() {
    let u = Speed::<f64>::new(1.0).unwrap();
    let length = Length::<f64>::new(0.1).unwrap();
    let effect = peclet_number(&u, &length, 0.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_strouhal_number_wrapper_error_path() {
    // u = 0 ⇒ division by zero.
    let length = Length::<f64>::new(0.1).unwrap();
    let u = Speed::<f64>::new(0.0).unwrap();
    let effect = strouhal_number(1.0_f64, &length, &u);
    assert!(!effect.is_ok());
}

#[test]
fn test_knudsen_number_wrapper_error_path() {
    // length = 0 ⇒ division by zero.
    let length = Length::<f64>::new(0.0).unwrap();
    let effect = knudsen_number(1.0e-7_f64, &length);
    assert!(!effect.is_ok());
}

#[test]
fn test_richardson_number_wrapper_error_path() {
    // u = 0 in denominator ⇒ division by zero.
    let u = Speed::<f64>::new(0.0).unwrap();
    let length = Length::<f64>::new(1.0).unwrap();
    let effect = richardson_number(9.81_f64, 3.4e-3_f64, 10.0_f64, &length, &u);
    assert!(!effect.is_ok());
}

#[test]
fn test_rayleigh_number_wrapper_error_path() {
    // thermal_diffusivity = 0 ⇒ division by zero.
    let nu = KinematicViscosity::<f64>::new(1.0e-6).unwrap();
    let length = Length::<f64>::new(0.1).unwrap();
    let effect = rayleigh_number(9.81_f64, 3.4e-3_f64, 10.0_f64, &length, &nu, 0.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_grashof_number_wrapper_error_path() {
    // nu = 0 ⇒ division by zero.
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    let length = Length::<f64>::new(0.1).unwrap();
    let effect = grashof_number(9.81_f64, 3.4e-3_f64, 10.0_f64, &length, &nu);
    assert!(!effect.is_ok());
}

#[test]
fn test_schmidt_number_wrapper_error_path() {
    let nu = KinematicViscosity::<f64>::new(1.0e-6).unwrap();
    let effect = schmidt_number(&nu, 0.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_lewis_number_wrapper_error_path() {
    // mass_diffusivity = 0 ⇒ division by zero.
    let effect = lewis_number(1.0e-7_f64, 0.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_capillary_number_wrapper_error_path() {
    let mu = Viscosity::<f64>::new(1.0e-3).unwrap();
    let u = Speed::<f64>::new(1.0).unwrap();
    let effect = capillary_number(&mu, &u, 0.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_bond_number_wrapper_error_path() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let length = Length::<f64>::new(0.01).unwrap();
    let effect = bond_number(&rho, 9.81_f64, &length, 0.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_nusselt_number_wrapper_error_path() {
    // thermal_conductivity = 0 ⇒ division by zero.
    let length = Length::<f64>::new(0.1).unwrap();
    let effect = nusselt_number(10.0_f64, &length, 0.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_particle_stokes_number_wrapper_error_path() {
    // length = 0 ⇒ division by zero.
    let u = Speed::<f64>::new(1.0).unwrap();
    let length = Length::<f64>::new(0.0).unwrap();
    let effect = particle_stokes_number(0.01_f64, &u, &length);
    assert!(!effect.is_ok());
}

#[test]
fn test_eckert_number_wrapper_error_path() {
    // c_p · ΔT = 0 ⇒ division by zero.
    let u = Speed::<f64>::new(100.0).unwrap();
    let effect = eckert_number(&u, 0.0_f64, 0.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_total_pressure_isentropic_wrapper_error_path() {
    // γ ≤ 1 errors.
    let p = Pressure::<f64>::new(101325.0).unwrap();
    let effect = total_pressure_isentropic(&p, 1.0_f64, 1.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_total_temperature_isentropic_wrapper_error_path() {
    // γ ≤ 1 errors.
    let t = Temperature::<f64>::new(300.0).unwrap();
    let effect = total_temperature_isentropic(&t, 1.0_f64, 1.0_f64);
    assert!(!effect.is_ok());
}

#[test]
fn test_wall_shear_stress_newtonian_wrapper_error_path() {
    // Non-finite gradient ⇒ WallShearStress::new rejects.
    let mu = Viscosity::<f64>::new(1.0e-3).unwrap();
    let effect = wall_shear_stress_newtonian(&mu, f64::NAN);
    assert!(!effect.is_ok());
}

#[test]
fn test_newtonian_viscous_stress_wrapper_error_path() {
    // Non-finite div_u ⇒ ViscousStress::new rejects.
    let mu = Viscosity::<f64>::new(1.0).unwrap();
    let s = StrainRateTensor::<f64>::default();
    let effect = newtonian_viscous_stress(&mu, &s, f64::NAN);
    assert!(!effect.is_ok());
}

#[test]
fn test_newtonian_viscous_stress_with_bulk_wrapper_error_path() {
    let mu = Viscosity::<f64>::new(1.0).unwrap();
    let zeta = Viscosity::<f64>::new(0.5).unwrap();
    let s = StrainRateTensor::<f64>::default();
    let effect = newtonian_viscous_stress_with_bulk(&mu, &zeta, &s, f64::INFINITY);
    assert!(!effect.is_ok());
}

// =============================================================================
// isentropic ratio + area–Mach wrapper tests
// =============================================================================

#[test]
fn test_isentropic_pressure_ratio_wrapper() {
    let effect = isentropic_pressure_ratio(2.0_f64, 1.4);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        isentropic_pressure_ratio_kernel(2.0_f64, 1.4).unwrap(),
        "isentropic_pressure_ratio must carry the value its kernel produced"
    );
    let v = effect.value_cloned().unwrap();
    assert!((v - 1.8_f64.powf(3.5)).abs() < 1e-9);
}

#[test]
fn test_isentropic_pressure_ratio_wrapper_error_path() {
    assert!(isentropic_pressure_ratio(f64::NAN, 1.4).is_err());
}

#[test]
fn test_isentropic_temperature_ratio_wrapper() {
    let effect = isentropic_temperature_ratio(2.0_f64, 1.4);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        isentropic_temperature_ratio_kernel(2.0_f64, 1.4).unwrap(),
        "isentropic_temperature_ratio must carry the value its kernel produced"
    );
    let v = effect.value_cloned().unwrap();
    assert!((v - 1.8).abs() < 1e-12);
}

#[test]
fn test_isentropic_temperature_ratio_wrapper_error_path() {
    assert!(isentropic_temperature_ratio(2.0_f64, 1.0).is_err());
}

#[test]
fn test_isentropic_density_ratio_wrapper() {
    let effect = isentropic_density_ratio(1.0_f64, 1.4);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        isentropic_density_ratio_kernel(1.0_f64, 1.4).unwrap(),
        "isentropic_density_ratio must carry the value its kernel produced"
    );
    let v = effect.value_cloned().unwrap();
    assert!((v - 1.2_f64.powf(2.5)).abs() < 1e-9);
}

#[test]
fn test_isentropic_density_ratio_wrapper_error_path() {
    assert!(isentropic_density_ratio(-1.0_f64, 1.4).is_err());
}

#[test]
fn test_area_mach_ratio_wrapper() {
    let effect = area_mach_ratio(2.0_f64, 1.4);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        area_mach_ratio_kernel(2.0_f64, 1.4).unwrap(),
        "area_mach_ratio must carry the value its kernel produced"
    );
    let v = effect.value_cloned().unwrap();
    assert!((v - 1.6875).abs() < 1e-9);
}

#[test]
fn test_area_mach_ratio_wrapper_error_path() {
    assert!(area_mach_ratio(0.0_f64, 1.4).is_err());
}

// =============================================================================
// Delegation, against the independent oracle
//
// Every wrapper below used to be tested by `assert!(f(..).is_ok())` and nothing else. That is
// satisfied by any implementation that returns `Ok` of anything: replacing `weber_number`'s body
// with `PropagatingEffect::pure(R::zero())` — reporting zero for every input in the world — passed
// the whole of this file.
//
// A wrapper's job is to carry the kernel's value into a `PropagatingEffect` and its error into a
// failed one, so the test is that the value arrives intact, over the same diverse inputs the
// kernels are checked on. The expectations come from `scripts/physics_oracles.py`, which evaluates
// the textbook definitions in 50-digit decimal, so this is not the kernel's formula retyped.
// =============================================================================

/// Relative agreement with the oracle; the wrapper adds no arithmetic of its own.
#[track_caller]
fn wrapper_close(got: f64, want: f64, case: &str) {
    let scale = want.abs().max(1.0);
    assert!(
        (got - want).abs() <= 1e-14 * scale,
        "{case}: wrapper gave {got:.17e}, oracle {want:.17e}"
    );
}

/// `(u, L, nu, expected)`
const REYNOLDS: &[[f64; 4]] = &[
    [1e-06, 1e-06, 1e-06, 1e-06],
    [0.001, 0.001, 1.5e-05, 0.06666666666666667],
    [1.0, 0.01, 1e-06, 10000.0],
    [10.0, 2.5, 1.5e-05, 1666666.6666666667],
    [250.0, 60.0, 1.46e-05, 1027397260.2739726],
    [7800.0, 10.0, 0.0001, 780000000.0],
    [100000.0, 1000.0, 1e-07, 1000000000000000.0],
    [0.037, 0.0089, 2.3e-06, 143.17391304347825],
];

/// `(u, a, expected)`
const MACH: &[[f64; 3]] = &[
    [0.001, 340.29, 2.9386699579770196e-06],
    [34.029, 340.29, 0.1],
    [340.29, 340.29, 1.0],
    [680.58, 340.29, 2.0],
    [1701.45, 340.29, 5.0],
    [7800.0, 295.0, 26.440677966101696],
    [10000.0, 0.01, 1000000.0],
];

/// `(u, g, L, expected)`
const FROUDE: &[[f64; 4]] = &[
    [0.001, 9.80665, 0.001, 0.01009809988551276],
    [0.5, 9.80665, 0.1, 0.5049049942756381],
    [10.0, 9.80665, 2.5, 2.0196199771025523],
    [3.0, 1.62, 100.0, 0.23570226039551584],
    [15.0, 24.79, 1.0, 3.012679939775216],
    [1000.0, 9.80665, 1000000.0, 0.3193299567810587],
    [2.7, 3.72, 0.35, 2.366237169223285],
];

/// `(rho, u, L, sigma, expected)`
const WEBER: &[[f64; 5]] = &[
    [1000.0, 2.0, 0.001, 0.072, 55.55555555555556],
    [1.225, 100.0, 0.05, 0.072, 8506.944444444445],
    [13546.0, 0.5, 0.002, 0.4865, 13.921891058581705],
    [789.0, 10.0, 0.0001, 0.0223, 353.8116591928251],
    [1000.0, 0.001, 1e-06, 0.072, 1.3888888888888889e-08],
    [1.0, 10000.0, 1000.0, 0.001, 100000000000000.0],
];

/// `(nu, alpha, expected)`
const PRANDTL: &[[f64; 3]] = &[
    [1.5e-05, 2.2e-05, 0.6818181818181818],
    [1e-06, 1.43e-07, 6.993006993006993],
    [1.1e-07, 4.3e-05, 0.0025581395348837207],
    [0.0009, 8.6e-08, 10465.116279069767],
    [1e-09, 1e-09, 1.0],
    [1000.0, 0.001, 1000000.0],
];

/// `(u, L, alpha, expected)`
const PECLET: &[[f64; 4]] = &[
    [1e-06, 1e-06, 1e-07, 1e-05],
    [1.0, 0.01, 1.43e-07, 69930.06993006993],
    [10.0, 2.5, 2.2e-05, 1136363.6363636365],
    [250.0, 60.0, 2.2e-05, 681818181.8181819],
    [10000.0, 1000.0, 1e-08, 1000000000000000.0],
];

/// `(f, L, u, expected)`
const STROUHAL: &[[f64; 4]] = &[
    [0.2, 1.0, 1.0, 0.2],
    [120.0, 0.01, 10.0, 0.12],
    [0.001, 1000.0, 1.0, 1.0],
    [5.0, 0.05, 1.2, 0.20833333333333334],
    [1000000.0, 1e-06, 0.001, 1000.0],
];

/// `(lam, L, expected)`
const KNUDSEN: &[[f64; 3]] = &[
    [6.8e-08, 1.0, 6.8e-08],
    [6.8e-08, 1e-06, 0.068],
    [0.001, 0.001, 1.0],
    [1.0, 1e-09, 1000000000.0],
    [1e-12, 1000.0, 1e-15],
];

/// `(g, beta, dT, L, u, expected)`
const RICHARDSON: &[[f64; 6]] = &[
    [9.80665, 0.0034, 10.0, 1.0, 1.0, 0.3334261],
    [9.80665, 0.000207, 50.0, 0.1, 0.01, 101.4988275],
    [1.62, 0.001, 100.0, 10.0, 5.0, 0.0648],
    [9.80665, 1e-06, 0.001, 0.001, 1000.0, 9.80665e-18],
];

/// `(g, beta, dT, L, nu, alpha, expected)`
const RAYLEIGH: &[[f64; 7]] = &[
    [
        9.80665,
        0.0034,
        10.0,
        0.1,
        1.5e-05,
        2.2e-05,
        1010382.1212121212,
    ],
    [
        9.80665,
        0.000207,
        20.0,
        0.05,
        1e-06,
        1.43e-07,
        35489100.52447552,
    ],
    [1.62, 0.001, 5.0, 2.0, 0.0001, 1e-05, 64800000.0],
    [9.80665, 1e-05, 0.01, 0.001, 0.001, 0.001, 9.80665e-10],
];

/// `(g, beta, dT, L, nu, expected)`
const GRASHOF: &[[f64; 6]] = &[
    [9.80665, 0.0034, 10.0, 0.1, 1.5e-05, 1481893.7777777778],
    [9.80665, 0.000207, 20.0, 0.05, 1e-06, 5074941.375],
    [1.62, 0.001, 5.0, 2.0, 0.0001, 6480000.0],
    [24.79, 0.0001, 1000.0, 100.0, 0.01, 24790000000.0],
];

/// `(u, cp, dT, expected)`
const ECKERT: &[[f64; 4]] = &[
    [2000.0, 1004.0, 3000.0, 1.3280212483399734],
    [10.0, 4184.0, 1.0, 0.02390057361376673],
    [0.001, 1000.0, 1000.0, 1e-12],
    [7800.0, 1004.0, 20000.0, 3.0298804780876494],
    [10000.0, 0.001, 0.001, 100000000000000.0],
];

/// `(nu, Dm, expected)`
const SCHMIDT: &[[f64; 3]] = &[
    [1.5e-05, 2e-05, 0.75],
    [1e-06, 1.5e-09, 666.6666666666666],
    [1e-09, 0.001, 1e-06],
    [0.0009, 1e-10, 9000000.0],
];

/// `(alpha, Dm, expected)`
const LEWIS: &[[f64; 3]] = &[
    [2.2e-05, 2e-05, 1.1],
    [1.43e-07, 1.5e-09, 95.33333333333333],
    [0.001, 1e-09, 1000000.0],
    [1e-09, 0.001, 1e-06],
];

/// `(tau, u, L, expected)`
const STOKES: &[[f64; 4]] = &[
    [0.001, 10.0, 0.1, 0.1],
    [1e-06, 1.0, 0.001, 0.001],
    [1.0, 1000.0, 0.01, 100000.0],
    [1e-09, 0.001, 1000.0, 1e-15],
];

/// `(mu, u, sigma, expected)`
const CAPILLARY: &[[f64; 4]] = &[
    [0.001, 0.01, 0.072, 0.0001388888888888889],
    [1.5e-05, 100.0, 0.072, 0.020833333333333332],
    [1.0, 1e-06, 0.0223, 4.484304932735426e-05],
    [0.1, 1.0, 0.4865, 0.20554984583761562],
    [1e-06, 1000000.0, 0.001, 1000.0],
];

/// `(rho, g, L, sigma, expected)`
const BOND: &[[f64; 5]] = &[
    [1000.0, 9.80665, 0.01, 0.072, 13.620347222222222],
    [1000.0, 9.80665, 0.001, 0.072, 0.13620347222222223],
    [13546.0, 9.80665, 0.005, 0.4865, 6.826355647482014],
    [789.0, 1.62, 0.02, 0.0223, 22.92699551569507],
    [1.0, 24.79, 1000.0, 0.001, 24790000000.0],
];

/// `(h, L, k, expected)`
const NUSSELT: &[[f64; 4]] = &[
    [10.0, 1.0, 0.0257, 389.10505836575874],
    [1000.0, 0.05, 0.6, 83.33333333333333],
    [0.001, 0.001, 1000.0, 1e-09],
    [100000.0, 10.0, 401.0, 2493.7655860349128],
    [25.0, 0.3, 0.14, 53.57142857142857],
];

#[test]
fn test_the_dimensionless_wrappers_carry_the_value_not_merely_an_ok() {
    // One assertion per row of every table: 100-odd inputs across all eighteen wrappers.
    for [u, l, v, want] in REYNOLDS {
        let e = reynolds_number(
            &Speed::new(*u).unwrap(),
            &Length::new(*l).unwrap(),
            &KinematicViscosity::new(*v).unwrap(),
        );
        wrapper_close(e.value_cloned().unwrap(), *want, "reynolds_number");
    }
    for [u, a, want] in MACH {
        let e = mach_number(&Speed::new(*u).unwrap(), &Speed::new(*a).unwrap());
        wrapper_close(e.value_cloned().unwrap(), *want, "mach_number");
    }
    for [u, g, l, want] in FROUDE {
        let e = froude_number(&Speed::new(*u).unwrap(), *g, &Length::new(*l).unwrap());
        wrapper_close(e.value_cloned().unwrap(), *want, "froude_number");
    }
    for [rho, u, l, s, want] in WEBER {
        let e = weber_number(
            &Density::new(*rho).unwrap(),
            &Speed::new(*u).unwrap(),
            &Length::new(*l).unwrap(),
            *s,
        );
        wrapper_close(e.value_cloned().unwrap(), *want, "weber_number");
    }
    for [v, alpha, want] in PRANDTL {
        let e = prandtl_number(&KinematicViscosity::new(*v).unwrap(), *alpha);
        wrapper_close(e.value_cloned().unwrap(), *want, "prandtl_number");
    }
    for [u, l, alpha, want] in PECLET {
        let e = peclet_number(&Speed::new(*u).unwrap(), &Length::new(*l).unwrap(), *alpha);
        wrapper_close(e.value_cloned().unwrap(), *want, "peclet_number");
    }
    for [f, l, u, want] in STROUHAL {
        let e = strouhal_number(*f, &Length::new(*l).unwrap(), &Speed::new(*u).unwrap());
        wrapper_close(e.value_cloned().unwrap(), *want, "strouhal_number");
    }
    for [lam, l, want] in KNUDSEN {
        let e = knudsen_number(*lam, &Length::new(*l).unwrap());
        wrapper_close(e.value_cloned().unwrap(), *want, "knudsen_number");
    }
    for [g, b, dt, l, u, want] in RICHARDSON {
        let e = richardson_number(
            *g,
            *b,
            *dt,
            &Length::new(*l).unwrap(),
            &Speed::new(*u).unwrap(),
        );
        wrapper_close(e.value_cloned().unwrap(), *want, "richardson_number");
    }
    for [g, b, dt, l, v, alpha, want] in RAYLEIGH {
        let e = rayleigh_number(
            *g,
            *b,
            *dt,
            &Length::new(*l).unwrap(),
            &KinematicViscosity::new(*v).unwrap(),
            *alpha,
        );
        wrapper_close(e.value_cloned().unwrap(), *want, "rayleigh_number");
    }
    for [g, b, dt, l, v, want] in GRASHOF {
        let e = grashof_number(
            *g,
            *b,
            *dt,
            &Length::new(*l).unwrap(),
            &KinematicViscosity::new(*v).unwrap(),
        );
        wrapper_close(e.value_cloned().unwrap(), *want, "grashof_number");
    }
    for [u, cp, dt, want] in ECKERT {
        let e = eckert_number(&Speed::new(*u).unwrap(), *cp, *dt);
        wrapper_close(e.value_cloned().unwrap(), *want, "eckert_number");
    }
    for [v, dm, want] in SCHMIDT {
        let e = schmidt_number(&KinematicViscosity::new(*v).unwrap(), *dm);
        wrapper_close(e.value_cloned().unwrap(), *want, "schmidt_number");
    }
    for [alpha, dm, want] in LEWIS {
        let e = lewis_number(*alpha, *dm);
        wrapper_close(e.value_cloned().unwrap(), *want, "lewis_number");
    }
    for [tau, u, l, want] in STOKES {
        let e = particle_stokes_number(*tau, &Speed::new(*u).unwrap(), &Length::new(*l).unwrap());
        wrapper_close(e.value_cloned().unwrap(), *want, "particle_stokes_number");
    }
    for [mu, u, s, want] in CAPILLARY {
        let e = capillary_number(&Viscosity::new(*mu).unwrap(), &Speed::new(*u).unwrap(), *s);
        wrapper_close(e.value_cloned().unwrap(), *want, "capillary_number");
    }
    for [rho, g, l, s, want] in BOND {
        let e = bond_number(
            &Density::new(*rho).unwrap(),
            *g,
            &Length::new(*l).unwrap(),
            *s,
        );
        wrapper_close(e.value_cloned().unwrap(), *want, "bond_number");
    }
    for [h, l, k, want] in NUSSELT {
        let e = nusselt_number(*h, &Length::new(*l).unwrap(), *k);
        wrapper_close(e.value_cloned().unwrap(), *want, "nusselt_number");
    }
}
