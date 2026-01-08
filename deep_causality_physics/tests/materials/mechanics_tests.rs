/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    PhysicsErrorEnum, Temperature, hookes_law_kernel, thermal_expansion_kernel,
    von_mises_stress_kernel,
};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// hookes_law_kernel Tests
// =============================================================================

#[test]
fn test_hookes_law_kernel_valid() {
    // σ_ij = C_ijkl × ε_kl
    // Create a simple rank-4 stiffness tensor (3x3x3x3)
    let mut stiffness_data = vec![0.0; 81];
    // Set C_0000 = 1.0, etc. for simple identity-like behavior
    stiffness_data[0] = 1.0; // C[0][0][0][0]
    let stiffness = CausalTensor::new(stiffness_data, vec![3, 3, 3, 3]).unwrap();

    // Create a strain tensor (3x3)
    let strain = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();

    let result = hookes_law_kernel(&stiffness, &strain);
    assert!(result.is_ok());

    let stress = result.unwrap();
    assert_eq!(stress.num_dim(), 2, "Result should be rank-2 tensor");
}

#[test]
fn test_hookes_law_kernel_dimension_mismatch_stiffness() {
    // Wrong rank stiffness (rank 2 instead of 4)
    let stiffness = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let strain = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();

    let result = hookes_law_kernel(&stiffness, &strain);
    assert!(result.is_err());

    match &result.unwrap_err().0 {
        PhysicsErrorEnum::DimensionMismatch(msg) => {
            assert!(msg.contains("Rank 4"));
        }
        _ => panic!("Expected DimensionMismatch error"),
    }
}

#[test]
fn test_hookes_law_kernel_dimension_mismatch_strain() {
    // Wrong rank strain (rank 1 instead of 2)
    let stiffness = CausalTensor::new(vec![0.0; 81], vec![3, 3, 3, 3]).unwrap();
    let strain = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();

    let result = hookes_law_kernel(&stiffness, &strain);
    assert!(result.is_err());
}

// =============================================================================
// von_mises_stress_kernel Tests
// =============================================================================

#[test]
fn test_von_mises_stress_kernel_valid() {
    // Create a 3x3 stress tensor
    // Uniaxial tension: only σ_xx = 100 MPa
    let mut stress_data = vec![0.0; 9];
    stress_data[0] = 100e6; // σ[0][0] = 100 MPa
    let stress = CausalTensor::new(stress_data, vec![3, 3]).unwrap();

    let result = von_mises_stress_kernel(&stress);
    assert!(result.is_ok());

    let vm_stress = result.unwrap();
    // For uniaxial stress: σ_vm = |σ_xx| = 100 MPa
    assert!(vm_stress.value() > 0.0);
}

#[test]
fn test_von_mises_stress_kernel_hydrostatic() {
    // Pure hydrostatic stress has zero deviatoric → zero Von Mises
    // σ = p * I (uniform pressure on diagonal)
    let p = 50e6;
    let stress =
        CausalTensor::new(vec![p, 0.0, 0.0, 0.0, p, 0.0, 0.0, 0.0, p], vec![3, 3]).unwrap();

    let result = von_mises_stress_kernel(&stress);
    assert!(result.is_ok());

    let vm_stress = result.unwrap();
    // For pure hydrostatic, deviatoric = 0, so Von Mises should be ~0
    assert!(
        vm_stress.value().abs() < 1e-6,
        "Hydrostatic stress should give near-zero Von Mises stress, got {}",
        vm_stress.value()
    );
}

#[test]
fn test_von_mises_stress_kernel_dimension_error() {
    // Wrong shape (2x2 instead of 3x3)
    let stress = CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap();

    let result = von_mises_stress_kernel(&stress);
    assert!(result.is_err());

    match &result.unwrap_err().0 {
        PhysicsErrorEnum::DimensionMismatch(msg) => {
            assert!(msg.contains("3x3"));
        }
        _ => panic!("Expected DimensionMismatch error"),
    }
}

#[test]
fn test_von_mises_stress_kernel_rank_error() {
    // Rank 3 tensor
    let stress = CausalTensor::new(vec![0.0; 27], vec![3, 3, 3]).unwrap();
    let result = von_mises_stress_kernel(&stress);
    assert!(result.is_err());
}

#[test]
fn test_von_mises_stress_kernel_shape_error() {
    // 3x4 tensor (Rank 2 but wrong shape)
    let stress = CausalTensor::new(vec![0.0; 12], vec![3, 4]).unwrap();
    let result = von_mises_stress_kernel(&stress);
    assert!(result.is_err());
}

// =============================================================================
// thermal_expansion_kernel Tests
// =============================================================================

#[test]
fn test_thermal_expansion_kernel_valid() {
    // ε = α × ΔT × I
    let alpha = 12e-6; // Thermal expansion coefficient (steel)
    let delta_temp = Temperature::new(100.0).unwrap(); // 100K increase

    let result = thermal_expansion_kernel(alpha, delta_temp);
    assert!(result.is_ok());

    let strain = result.unwrap();
    // Should be 3x3 isotropic strain tensor
    assert_eq!(strain.shape(), vec![3, 3]);

    // Diagonal elements should be α * ΔT
    let expected = alpha * 100.0;
    assert!(
        (strain.data()[0] - expected).abs() < 1e-15,
        "Expected diagonal strain {}, got {}",
        expected,
        strain.data()[0]
    );
}

#[test]
fn test_thermal_expansion_kernel_zero_temp() {
    let alpha = 12e-6;
    let delta_temp = Temperature::new(0.0).unwrap();

    let result = thermal_expansion_kernel(alpha, delta_temp);
    assert!(result.is_ok());

    let strain = result.unwrap();
    // Zero temperature change → zero strain
    assert!(
        strain.data()[0].abs() < 1e-15,
        "Zero ΔT should give zero strain"
    );
}
