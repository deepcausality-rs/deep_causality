/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AmountOfSubstance, Energy, Pressure, Temperature, Volume, boltzmann_factor, carnot_efficiency,
    heat_capacity, heat_diffusion, ideal_gas_law, partition_function, shannon_entropy,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialManifold};

// Helper — manifold with a unit-edge metric attached (required by R4.5:
// `heat_diffusion` wrapper calls `manifold.laplacian()`).
fn create_temp_manifold() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(
        vec![
            0.0, 0.0, // v0
            1.0, 0.0, // v1
            0.5, 0.866, // v2
        ],
        vec![3, 2],
    )
    .unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num_simplices = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    let initial_data = vec![300.0; num_simplices];
    let metric =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(initial_data, vec![num_simplices]).unwrap(),
        Some(metric),
        0,
    )
    .unwrap()
}

#[test]
fn test_ideal_gas_law_wrapper_success() {
    let p = Pressure::<f64>::new(101325.0).unwrap();
    let v = Volume::<f64>::new(0.0224).unwrap();
    let n = AmountOfSubstance::<f64>::new(1.0).unwrap();
    let t = Temperature::<f64>::new(273.15).unwrap();

    let effect = ideal_gas_law(p, v, n, t);
    assert!(effect.is_ok());
}

#[test]
fn test_ideal_gas_law_wrapper_error() {
    let p = Pressure::<f64>::new(100.0).unwrap();
    let v = Volume::<f64>::new(1.0).unwrap();
    let n = AmountOfSubstance::<f64>::new(0.0).unwrap(); // Zero moles
    let t = Temperature::<f64>::new(300.0).unwrap();

    let effect = ideal_gas_law(p, v, n, t);
    assert!(effect.is_err());
}

#[test]
fn test_carnot_efficiency_wrapper_success() {
    let th = Temperature::<f64>::new(500.0).unwrap();
    let tc = Temperature::<f64>::new(300.0).unwrap();

    let effect = carnot_efficiency(th, tc);
    assert!(effect.is_ok());

    let eff = effect.value().clone().into_value().unwrap();
    assert!((eff.value() - 0.4).abs() < 1e-10);
}

#[test]
fn test_carnot_efficiency_wrapper_error() {
    let th = Temperature::<f64>::new(300.0).unwrap();
    let tc = Temperature::<f64>::new(300.0).unwrap(); // Tc >= Th

    let effect = carnot_efficiency(th, tc);
    assert!(effect.is_err());
}

#[test]
fn test_boltzmann_factor_wrapper_success() {
    let e = Energy::<f64>::new(0.0).unwrap();
    let t = Temperature::<f64>::new(300.0).unwrap();

    let effect = boltzmann_factor(e, t);
    assert!(effect.is_ok());
}

#[test]
fn test_boltzmann_factor_wrapper_error() {
    let e = Energy::<f64>::new(0.0).unwrap();
    let t = Temperature::<f64>::new(0.0).unwrap(); // T=0
    let effect = boltzmann_factor(e, t);
    assert!(effect.is_err());
}

#[test]
fn test_shannon_entropy_wrapper_success() {
    let probs = CausalTensor::new(vec![0.25, 0.25, 0.25, 0.25], vec![4]).unwrap();

    let effect = shannon_entropy(&probs);
    assert!(effect.is_ok());
}

#[test]
fn test_shannon_entropy_wrapper_error() {
    let probs = CausalTensor::new(vec![-0.1], vec![1]).unwrap(); // Negative prob
    let effect = shannon_entropy(&probs);
    assert!(effect.is_err());
}

#[test]
fn test_heat_capacity_wrapper_success() {
    let de = Energy::<f64>::new(100.0).unwrap();
    let dt = Temperature::<f64>::new(10.0).unwrap();

    let effect = heat_capacity(de, dt);
    assert!(effect.is_ok());

    let c = effect.value().clone().into_value().unwrap();
    assert!((c - 10.0).abs() < 1e-10);
}

#[test]
fn test_heat_capacity_wrapper_error() {
    let de = Energy::<f64>::new(100.0).unwrap();
    let dt = Temperature::<f64>::new(0.0).unwrap(); // Zero dT

    let effect = heat_capacity(de, dt);
    assert!(effect.is_err());
}

#[test]
fn test_partition_function_wrapper_success() {
    let energies = CausalTensor::new(vec![0.0, 1e-21, 2e-21], vec![3]).unwrap();
    let t = Temperature::<f64>::new(300.0).unwrap();

    let effect = partition_function(&energies, t);
    assert!(effect.is_ok());
}

#[test]
fn test_partition_function_wrapper_error() {
    let energies = CausalTensor::new(vec![0.0], vec![1]).unwrap();
    let t = Temperature::<f64>::new(0.0).unwrap(); // T=0
    let effect = partition_function(&energies, t);
    assert!(effect.is_err());
}

#[test]
fn test_heat_diffusion_wrapper_success() {
    let manifold = create_temp_manifold();
    let effect = heat_diffusion(&manifold, 0.5);
    assert!(effect.is_ok());
}

#[test]
fn test_heat_diffusion_wrapper_error() {
    let manifold = create_temp_manifold();
    let effect = heat_diffusion(&manifold, -0.5); // Negative diffusivity
    assert!(effect.is_err());
}

// NOTE on thermodynamics/wrappers.rs:44, 61 — the `ok_or_else` closure bodies
// for `R::from_f64(BOLTZMANN_CONSTANT)` inside the `boltzmann_factor` /
// `partition_function` wrappers' underlying kernels (mirrored here at the
// wrapper layer). `from_f64` is infallible for f64, so the conversion never
// returns `None` and these defensive error closures can never run for the f64
// monomorphisation exercised by this suite.
