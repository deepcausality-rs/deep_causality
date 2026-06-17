/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the theory-layer causal wrappers (the NS-regime `PropagatingEffect`
//! wrappers), migrated out of `deep_causality_physics`'s `wrappers_tests.rs`.

use deep_causality_cfd::{
    AccelerationVector, Density, KinematicViscosity, Velocity3, VelocityGradient,
    compressible_ns_continuity_rhs_effect, compressible_ns_energy_rhs_effect,
    compressible_ns_momentum_rhs_effect, euler_momentum_rhs_effect, incompressible_ns_rhs_effect,
    stokes_momentum_rhs_effect,
};

#[test]
fn test_incompressible_ns_rhs_effect_wrapper() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g = VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap();
    let lap = [0.0_f64; 3];
    let gp = [10.0_f64, 0.0, 0.0];
    let r = Density::<f64>::new(2.0).unwrap();
    let n = KinematicViscosity::<f64>::new(0.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0, -9.81, 0.0]).unwrap();
    let effect = incompressible_ns_rhs_effect(&u, &g, &lap, &gp, &r, &n, &b);
    assert!(effect.is_ok());
    let v = effect.value().clone().into_value().unwrap().into_inner();
    assert!((v[0] - (-5.0)).abs() < 1e-12);
    assert!((v[1] - (-9.81)).abs() < 1e-12);
    assert!(v[2].abs() < 1e-12);
}

#[test]
fn test_euler_momentum_rhs_effect_wrapper() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g = VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap();
    let gp = [10.0_f64, 0.0, 0.0];
    let r = Density::<f64>::new(2.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0, -9.81, 0.0]).unwrap();
    let effect = euler_momentum_rhs_effect(&u, &g, &gp, &r, &b);
    assert!(effect.is_ok());
    let v = effect.value().clone().into_value().unwrap().into_inner();
    assert!((v[0] - (-5.0)).abs() < 1e-12);
    assert!((v[1] - (-9.81)).abs() < 1e-12);
}

#[test]
fn test_euler_momentum_rhs_effect_wrapper_error_path() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g = VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap();
    let gp = [1.0_f64, 0.0, 0.0];
    let r = Density::<f64>::new(0.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0; 3]).unwrap();
    let effect = euler_momentum_rhs_effect(&u, &g, &gp, &r, &b);
    assert!(!effect.is_ok());
}

#[test]
fn test_compressible_ns_continuity_rhs_effect_wrapper() {
    let rho = Density::<f64>::new(2.0).unwrap();
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let effect = compressible_ns_continuity_rhs_effect(&rho, &u, &[3.0, 0.0, 0.0], 0.5);
    assert!(effect.is_ok());
    assert!((effect.value().clone().into_value().unwrap() - (-4.0)).abs() < 1e-12);
}

#[test]
fn test_compressible_ns_momentum_rhs_effect_wrapper() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g = VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap();
    let gp = [10.0_f64, 0.0, 0.0];
    let div_tau = [0.0_f64; 3];
    let r = Density::<f64>::new(2.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0, -9.81, 0.0]).unwrap();
    let effect = compressible_ns_momentum_rhs_effect(&u, &g, &gp, &div_tau, &r, &b);
    assert!(effect.is_ok());
}

#[test]
fn test_compressible_ns_momentum_rhs_effect_wrapper_error_path() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g = VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap();
    let gp = [1.0_f64, 0.0, 0.0];
    let div_tau = [0.0_f64; 3];
    let r = Density::<f64>::new(0.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0; 3]).unwrap();
    let effect = compressible_ns_momentum_rhs_effect(&u, &g, &gp, &div_tau, &r, &b);
    assert!(!effect.is_ok());
}

#[test]
fn test_compressible_ns_energy_rhs_effect_wrapper() {
    let rho = Density::<f64>::new(1.0).unwrap();
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let b = AccelerationVector::<f64>::new([6.0, 0.0, 0.0]).unwrap();
    let effect = compressible_ns_energy_rhs_effect(&rho, &u, 2.0, 3.0, 4.0, 5.0, &b);
    assert!(effect.is_ok());
    assert!(effect.value().clone().into_value().unwrap().abs() < 1e-12);
}

#[test]
fn test_stokes_momentum_rhs_effect_wrapper() {
    let lap = [4.0_f64, 0.0, 0.0];
    let gp = [10.0_f64, 0.0, 0.0];
    let r = Density::<f64>::new(2.0).unwrap();
    let n = KinematicViscosity::<f64>::new(0.5).unwrap();
    let b = AccelerationVector::<f64>::new([0.0, -9.81, 0.0]).unwrap();
    let effect = stokes_momentum_rhs_effect(&lap, &gp, &r, &n, &b);
    assert!(effect.is_ok());
    let v = effect.value().clone().into_value().unwrap().into_inner();
    assert!((v[0] - (-3.0)).abs() < 1e-12);
    assert!((v[1] - (-9.81)).abs() < 1e-12);
}

#[test]
fn test_stokes_momentum_rhs_effect_wrapper_error_path() {
    let lap = [0.0_f64; 3];
    let gp = [1.0_f64, 0.0, 0.0];
    let r = Density::<f64>::new(0.0).unwrap();
    let n = KinematicViscosity::<f64>::new(0.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0; 3]).unwrap();
    let effect = stokes_momentum_rhs_effect(&lap, &gp, &r, &n, &b);
    assert!(!effect.is_ok());
}

#[test]
fn test_incompressible_ns_rhs_effect_wrapper_error_path() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g = VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap();
    let lap = [0.0_f64; 3];
    let gp = [1.0_f64, 0.0, 0.0];
    let r = Density::<f64>::new(0.0).unwrap();
    let n = KinematicViscosity::<f64>::new(0.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0; 3]).unwrap();
    let effect = incompressible_ns_rhs_effect(&u, &g, &lap, &gp, &r, &n, &b);
    assert!(!effect.is_ok());
}
