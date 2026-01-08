/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AmountOfSubstance, Energy, Pressure, Temperature, Volume, boltzmann_factor_kernel,
    carnot_efficiency_kernel, heat_capacity_kernel, ideal_gas_law_kernel,
    partition_function_kernel, shannon_entropy_kernel,
};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// ideal_gas_law_kernel Tests
// =============================================================================

#[test]
fn test_ideal_gas_law_kernel_valid() {
    // PV = nRT => R = PV/nT
    // Use values that give R ~ 8.314 J/(mol·K)
    let p = Pressure::new(101325.0).unwrap(); // 1 atm in Pa
    let v = Volume::new(0.0224).unwrap(); // ~22.4 L = 0.0224 m³
    let n = AmountOfSubstance::new(1.0).unwrap(); // 1 mol
    let t = Temperature::new(273.15).unwrap(); // 0°C = 273.15K

    let result = ideal_gas_law_kernel(p, v, n, t);
    assert!(result.is_ok());

    let r = result.unwrap();
    // R should be close to 8.314
    assert!((r - 8.314).abs() < 0.1, "Expected R ~ 8.314, got {}", r);
}

#[test]
fn test_ideal_gas_law_kernel_zero_moles_error() {
    let p = Pressure::new(100.0).unwrap();
    let v = Volume::new(1.0).unwrap();
    let n = AmountOfSubstance::new(0.0).unwrap();
    let t = Temperature::new(300.0).unwrap();

    let result = ideal_gas_law_kernel(p, v, n, t);
    assert!(result.is_err(), "Zero moles should error");
}

// =============================================================================
// carnot_efficiency_kernel Tests
// =============================================================================

#[test]
fn test_carnot_efficiency_kernel_valid() {
    // η = 1 - Tc/Th
    let th = Temperature::new(500.0).unwrap();
    let tc = Temperature::new(300.0).unwrap();

    let result = carnot_efficiency_kernel(th, tc);
    assert!(result.is_ok());

    let eff = result.unwrap();
    // η = 1 - 300/500 = 0.4
    assert!((eff - 0.4).abs() < 1e-10, "Expected 0.4, got {}", eff);
}

#[test]
fn test_carnot_efficiency_kernel_cold_ge_hot_error() {
    let th = Temperature::new(300.0).unwrap();
    let tc = Temperature::new(300.0).unwrap();

    let result = carnot_efficiency_kernel(th, tc);
    assert!(result.is_err(), "Tc >= Th should error");
}

// =============================================================================
// boltzmann_factor_kernel Tests
// =============================================================================

#[test]
fn test_boltzmann_factor_kernel_ground_state() {
    // E=0 => factor = 1
    let e = Energy::new(0.0).unwrap();
    let t = Temperature::new(300.0).unwrap();

    let result = boltzmann_factor_kernel(e, t);
    assert!(result.is_ok());

    let p = result.unwrap();
    assert!((p.value() - 1.0).abs() < 1e-10);
}

// =============================================================================
// shannon_entropy_kernel Tests
// =============================================================================

#[test]
fn test_shannon_entropy_kernel_uniform() {
    // Uniform distribution has max entropy
    let probs = CausalTensor::new(vec![0.25, 0.25, 0.25, 0.25], vec![4]).unwrap();

    let result = shannon_entropy_kernel(&probs);
    assert!(result.is_ok());

    let h = result.unwrap();
    // H = -4 * (0.25 * ln(0.25)) = ln(4) ≈ 1.386
    assert!((h - 1.386).abs() < 0.01, "Expected ~1.386, got {}", h);
}

// =============================================================================
// heat_capacity_kernel Tests
// =============================================================================

#[test]
fn test_heat_capacity_kernel_valid() {
    let de = Energy::new(100.0).unwrap();
    let dt = Temperature::new(10.0).unwrap();

    let result = heat_capacity_kernel(de, dt);
    assert!(result.is_ok());

    let c = result.unwrap();
    assert!((c - 10.0).abs() < 1e-10);
}

#[test]
fn test_heat_capacity_kernel_zero_dt_error() {
    let de = Energy::new(100.0).unwrap();
    let dt = Temperature::new(0.0).unwrap();

    let result = heat_capacity_kernel(de, dt);
    assert!(result.is_err(), "Zero dT should error");
}

// =============================================================================
// partition_function_kernel Tests
// =============================================================================

#[test]
fn test_partition_function_kernel_valid() {
    let energies = CausalTensor::new(vec![0.0, 1e-21, 2e-21], vec![3]).unwrap();
    let t = Temperature::new(300.0).unwrap();

    let result = partition_function_kernel(&energies, t);
    assert!(result.is_ok());

    let z = result.unwrap();
    assert!(z > 0.0);
}

#[test]
fn test_partition_function_kernel_error() {
    let energies = CausalTensor::new(vec![0.0], vec![1]).unwrap();
    let t = Temperature::new(0.0).unwrap(); // T=0 error

    let result = partition_function_kernel(&energies, t);
    assert!(result.is_err());
}

// =============================================================================
// boltzmann_factor_kernel Tests (Error Case)
// =============================================================================

#[test]
fn test_boltzmann_factor_kernel_error() {
    let e = Energy::new(0.0).unwrap();
    let t = Temperature::new(0.0).unwrap(); // T=0 error

    let result = boltzmann_factor_kernel(e, t);
    assert!(result.is_err());
}

// =============================================================================
// shannon_entropy_kernel Tests (Error Case)
// =============================================================================

#[test]
fn test_shannon_entropy_kernel_error() {
    // Negative probability
    let probs = CausalTensor::new(vec![0.5, -0.1], vec![2]).unwrap();
    let result = shannon_entropy_kernel(&probs);
    assert!(result.is_err());

    // Empty tensor
    let empty_probs = CausalTensor::new(vec![], vec![0]).unwrap();
    let result_empty = shannon_entropy_kernel(&empty_probs);
    assert!(result_empty.is_err());
}

// =============================================================================
// heat_diffusion_kernel Tests
// =============================================================================
use deep_causality_physics::heat_diffusion_kernel;
use deep_causality_topology::{Manifold, PointCloud};

// Helper to create a simple manifold for heat diffusion (Vertices only for 0-form)
// Using same structure as creating a simple manifold in other tests
fn create_temp_manifold() -> Manifold<f64, f64> {
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
    // Initialize with dummy temp data
    let initial_data = vec![300.0; num_simplices];
    Manifold::new(
        complex,
        CausalTensor::new(initial_data, vec![num_simplices]).unwrap(),
        0,
    )
    .unwrap()
}

#[test]
fn test_heat_diffusion_kernel_valid() {
    let manifold = create_temp_manifold();
    let diffusivity = 0.5;
    let result = heat_diffusion_kernel(&manifold, diffusivity);
    assert!(result.is_ok());
}

#[test]
fn test_heat_diffusion_kernel_negative_diffusivity_error() {
    let manifold = create_temp_manifold();
    let diffusivity = -0.5; // Invalid
    let result = heat_diffusion_kernel(&manifold, diffusivity);
    assert!(result.is_err());
}
