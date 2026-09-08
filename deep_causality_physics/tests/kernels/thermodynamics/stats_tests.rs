/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AmountOfSubstance, Energy, Pressure, Temperature, Volume, boltzmann_factor_kernel,
    carnot_efficiency_kernel, heat_capacity_kernel, ideal_gas_law_kernel,
    partition_function_kernel, shannon_entropy_bits_kernel,
};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// ideal_gas_law_kernel Tests
// =============================================================================

#[test]
fn test_ideal_gas_law_kernel_valid() {
    // PV = nRT => R = PV/nT
    // Use values that give R ~ 8.314 J/(mol·K)
    let p = Pressure::<f64>::new(101325.0).unwrap(); // 1 atm in Pa
    let v = Volume::<f64>::new(0.0224).unwrap(); // ~22.4 L = 0.0224 m³
    let n = AmountOfSubstance::<f64>::new(1.0).unwrap(); // 1 mol
    let t = Temperature::<f64>::new(273.15).unwrap(); // 0°C = 273.15K

    let result = ideal_gas_law_kernel(p, v, n, t);
    assert!(result.is_ok());

    let r = result.unwrap();
    // R should be close to 8.314
    assert!((r - 8.314).abs() < 0.1, "Expected R ~ 8.314, got {}", r);
}

#[test]
fn test_ideal_gas_law_kernel_zero_moles_error() {
    let p = Pressure::<f64>::new(100.0).unwrap();
    let v = Volume::<f64>::new(1.0).unwrap();
    let n = AmountOfSubstance::<f64>::new(0.0).unwrap();
    let t = Temperature::<f64>::new(300.0).unwrap();

    let result = ideal_gas_law_kernel(p, v, n, t);
    assert!(result.is_err(), "Zero moles should error");
}

// =============================================================================
// carnot_efficiency_kernel Tests
// =============================================================================

#[test]
fn test_carnot_efficiency_kernel_valid() {
    // η = 1 - Tc/Th
    let th = Temperature::<f64>::new(500.0).unwrap();
    let tc = Temperature::<f64>::new(300.0).unwrap();

    let result = carnot_efficiency_kernel(th, tc);
    assert!(result.is_ok());

    let eff = result.unwrap();
    // η = 1 - 300/500 = 0.4
    assert!((eff - 0.4).abs() < 1e-10, "Expected 0.4, got {}", eff);
}

#[test]
fn test_carnot_efficiency_kernel_cold_ge_hot_error() {
    let th = Temperature::<f64>::new(300.0).unwrap();
    let tc = Temperature::<f64>::new(300.0).unwrap();

    let result = carnot_efficiency_kernel(th, tc);
    assert!(result.is_err(), "Tc >= Th should error");
}

// =============================================================================
// boltzmann_factor_kernel Tests
// =============================================================================

#[test]
fn test_boltzmann_factor_kernel_ground_state() {
    // E=0 => factor = 1
    let e = Energy::<f64>::new(0.0).unwrap();
    let t = Temperature::<f64>::new(300.0).unwrap();

    let result = boltzmann_factor_kernel(e, t);
    assert!(result.is_ok());

    let p = result.unwrap();
    assert!((p.value() - 1.0).abs() < 1e-10);
}

// =============================================================================
// shannon_entropy_bits_kernel Tests
// =============================================================================

#[test]
fn test_shannon_entropy_bits_kernel_uniform() {
    // Uniform distribution has max entropy
    let probs: CausalTensor<f64> =
        CausalTensor::new(vec![0.25, 0.25, 0.25, 0.25], vec![4]).unwrap();

    let result = shannon_entropy_bits_kernel(&probs);
    assert!(result.is_ok());

    let h = result.unwrap();
    // Four equiprobable outcomes carry log2(4) = 2 bits.
    assert!((h - 2.0).abs() < 1e-14, "Expected 2 bits, got {}", h);
}

#[test]
fn test_shannon_entropy_bits_kernel_does_not_renormalise_its_input() {
    // The kernel's contract is `Normalisation::None`: the caller states that the input is a
    // distribution, and the kernel measures what it is given. Pinned because every other fixture in
    // this file already sums to one, where normalising and not normalising are the same operation —
    // so nothing here could tell a `BySum` delegation from a `None` one.
    //
    // `p = (0.5, 0.25)` sums to 0.75. Measured as given:
    //   H = −(0.5·log₂0.5 + 0.25·log₂0.25) = −(0.5·(−1) + 0.25·(−2)) = 0.5 + 0.5 = 1 bit exactly.
    // Normalised first it would be (2/3, 1/3), whose entropy is 0.9183 bits — a different number,
    // and a different claim about the caller's data.
    let probs: CausalTensor<f64> = CausalTensor::new(vec![0.5, 0.25], vec![2]).unwrap();
    let h = shannon_entropy_bits_kernel(&probs).expect("a sub-unit sum is not an error here");
    assert!((h - 1.0).abs() < 1e-15, "expected exactly 1 bit, got {h}");
}

#[test]
fn test_shannon_entropy_bits_kernel_counts_a_probability_far_below_epsilon() {
    // The other half of the config: `ZeroPolicy::SkipZero` skips an entry that is **exactly** zero
    // and nothing else. A threshold policy — `SkipBelow(ε)`, which the SURD path deliberately does
    // use — would drop this entry and report zero, and no other fixture in this file has an entry
    // between zero and ε to tell the two apart.
    //
    // p = (1, 1e-20): the first term is −1·log₂1 = 0, so the whole entropy is the second,
    // −1e-20·log₂(1e-20) = 1e-20 · 66.44 ≈ 6.64e-19. Small, but not zero — and it is the answer.
    let probs: CausalTensor<f64> = CausalTensor::new(vec![1.0, 1e-20], vec![2]).unwrap();
    let h = shannon_entropy_bits_kernel(&probs).expect("a tiny positive probability is admissible");
    assert!(h > 0.0, "a positive probability must contribute, got {h}");
    assert!(
        (h - 6.6438e-19).abs() < 1e-22,
        "expected about 6.644e-19 bits, got {h}"
    );
}

// =============================================================================
// heat_capacity_kernel Tests
// =============================================================================

#[test]
fn test_heat_capacity_kernel_valid() {
    let de = Energy::<f64>::new(100.0).unwrap();
    let dt = Temperature::<f64>::new(10.0).unwrap();

    let result = heat_capacity_kernel(de, dt);
    assert!(result.is_ok());

    let c = result.unwrap();
    assert!((c - 10.0).abs() < 1e-10);
}

#[test]
fn test_heat_capacity_kernel_zero_dt_error() {
    let de = Energy::<f64>::new(100.0).unwrap();
    let dt = Temperature::<f64>::new(0.0).unwrap();

    let result = heat_capacity_kernel(de, dt);
    assert!(result.is_err(), "Zero dT should error");
}

// =============================================================================
// partition_function_kernel Tests
// =============================================================================

#[test]
fn test_partition_function_kernel_valid() {
    let energies = CausalTensor::new(vec![0.0, 1e-21, 2e-21], vec![3]).unwrap();
    let t = Temperature::<f64>::new(300.0).unwrap();

    let result = partition_function_kernel(&energies, t);
    assert!(result.is_ok());

    let z = result.unwrap();
    assert!(z > 0.0);
}

#[test]
fn test_partition_function_kernel_error() {
    let energies = CausalTensor::new(vec![0.0], vec![1]).unwrap();
    let t = Temperature::<f64>::new(0.0).unwrap(); // T=0 error

    let result = partition_function_kernel(&energies, t);
    assert!(result.is_err());
}

// =============================================================================
// boltzmann_factor_kernel Tests (Error Case)
// =============================================================================

#[test]
fn test_boltzmann_factor_kernel_error() {
    let e = Energy::<f64>::new(0.0).unwrap();
    let t = Temperature::<f64>::new(0.0).unwrap(); // T=0 error

    let result = boltzmann_factor_kernel(e, t);
    assert!(result.is_err());
}

// =============================================================================
// shannon_entropy_bits_kernel Tests (Error Case)
// =============================================================================

#[test]
fn test_shannon_entropy_bits_kernel_error() {
    // Negative probability
    let probs = CausalTensor::new(vec![0.5, -0.1], vec![2]).unwrap();
    let result = shannon_entropy_bits_kernel(&probs);
    assert!(result.is_err());

    // Empty tensor
    let empty_probs: CausalTensor<f64> = CausalTensor::new(vec![], vec![0]).unwrap();
    let result_empty = shannon_entropy_bits_kernel(&empty_probs);
    assert!(result_empty.is_err());
}

// =============================================================================
// heat_diffusion_kernel Tests
// =============================================================================
use deep_causality_physics::heat_diffusion_kernel;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialManifold};

// Helper to create a simple manifold for heat diffusion (Vertices only for 0-form)
// Using same structure as creating a simple manifold in other tests
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
    // Initialize with dummy temp data
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
