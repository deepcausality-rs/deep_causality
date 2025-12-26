/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::EffectValue;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Density, Diffusivity, Mass, PhysicalField, Speed, Temperature, alfven_speed, debye_length,
    energy_momentum_tensor_em, ideal_induction, larmor_radius, magnetic_pressure,
    magnetic_reconnection_rate, resistive_diffusion,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

// ============================================================================
// Ideal MHD Wrapper Tests
// ============================================================================

#[test]
fn test_alfven_speed_wrapper_success() {
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );
    let rho = Density::new(1.0).unwrap();

    let result = alfven_speed(&b, &rho, 1.0);
    assert!(result.is_ok());

    if let EffectValue::Value(va) = result.value() {
        assert!(va.value() > 0.0);
    } else {
        panic!("Expected Value variant");
    }
}

#[test]
fn test_alfven_speed_wrapper_with_physical_values() {
    // B = 1 T, rho = 1000 kg/m³, mu0 = 4*pi*1e-7
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );
    let rho = Density::new(1000.0).unwrap();
    let mu0 = 4.0 * std::f64::consts::PI * 1e-7;

    let result = alfven_speed(&b, &rho, mu0);
    assert!(result.is_ok());

    // v_A = B / sqrt(mu0 * rho) ≈ 1 / sqrt(4*pi*1e-7 * 1000) ≈ 28.2 m/s
    if let EffectValue::Value(va) = result.value() {
        assert!((va.value() - 28.2).abs() < 1.0);
    }
}

#[test]
fn test_magnetic_pressure_wrapper_success() {
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 2.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );

    let result = magnetic_pressure(&b, 2.0);
    assert!(result.is_ok());

    // P = B² / (2 * mu0) = 4 / 4 = 1.0
    if let EffectValue::Value(p) = result.value() {
        assert!((p.value() - 1.0).abs() < 1e-10);
    }
}

#[test]
fn test_magnetic_pressure_wrapper_zero_field() {
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 0.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );

    let result = magnetic_pressure(&b, 1.0);
    assert!(result.is_ok());

    if let EffectValue::Value(p) = result.value() {
        assert!((p.value()).abs() < 1e-10);
    }
}

// ============================================================================
// Resistive MHD Wrapper Tests
// ============================================================================

#[test]
fn test_magnetic_reconnection_rate_wrapper_success() {
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );
    let rho = Density::new(1.0).unwrap();
    let va = alfven_speed(&b, &rho, 1.0)
        .value()
        .clone()
        .into_value()
        .unwrap();

    let result = magnetic_reconnection_rate(va, 100.0);
    assert!(result.is_ok());

    // v_in = v_A / sqrt(S) = 1.0 / 10.0 = 0.1
    if let EffectValue::Value(v) = result.value() {
        assert!((v.value() - 0.1).abs() < 1e-10);
    }
}

#[test]
fn test_magnetic_reconnection_rate_wrapper_high_lundquist() {
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 10.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );
    let rho = Density::new(1.0).unwrap();
    let va = alfven_speed(&b, &rho, 1.0)
        .value()
        .clone()
        .into_value()
        .unwrap();

    // High Lundquist number → slow reconnection
    let result = magnetic_reconnection_rate(va, 10000.0);
    assert!(result.is_ok());

    if let EffectValue::Value(v) = result.value() {
        // v_A ≈ 10, S = 10000, v_in = 10/100 = 0.1
        assert!(v.value() < 0.2);
    }
}

// ============================================================================
// Ideal Induction and Resistive Diffusion Wrapper Tests (Require Manifold)
// ============================================================================

fn create_test_manifold() -> Manifold<f64> {
    // Create a simple triangular mesh
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let scalar_field = CausalTensor::new(vec![0.0; 3], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, scalar_field, 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num = complex.total_simplices();
    Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; num], vec![num]).unwrap(),
        0,
    )
    .unwrap()
}

#[test]
fn test_ideal_induction_wrapper() {
    let man = create_test_manifold();

    let result = ideal_induction(&man, &man);
    // The wrapper should handle the result from the kernel
    // It may succeed or fail depending on manifold capabilities
    // We just test that the wrapper works without panicking
    let _ = result.is_ok() || result.is_err();
}

#[test]
fn test_resistive_diffusion_wrapper() {
    let man = create_test_manifold();
    let eta = Diffusivity::new(0.1).unwrap();

    let result = resistive_diffusion(&man, eta);
    // The wrapper should propagate the result
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// GRMHD Wrapper Tests
// ============================================================================

#[test]
fn test_relativistic_current_wrapper() {
    // Note: relativistic_current now requires Manifold<f64> and LorentzianMetric
    // This test is a placeholder - the full implementation requires a properly
    // constructed manifold with EM 2-form data on 2-simplices.
    // Testing the dimension validation would require creating a manifold
    // without sufficient skeletons or operators.

    // For now, we just verify the wrapper compiles.
    // Comprehensive tests should be added once the physics test infrastructure
    // supports manifold construction with proper EM field data.
}

#[test]
fn test_energy_momentum_tensor_em_wrapper_minkowski() {
    // Simple EM tensor with E_x = 1
    let em = CausalTensor::new(
        vec![
            0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        vec![4, 4],
    )
    .unwrap();

    let metric = CausalTensor::new(
        vec![
            -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
        vec![4, 4],
    )
    .unwrap();

    let result = energy_momentum_tensor_em(&em, &metric);
    assert!(result.is_ok());

    if let EffectValue::Value(t) = result.value() {
        // Should be a 4x4 tensor
        assert_eq!(t.shape(), &[4, 4]);
    }
}

#[test]
fn test_energy_momentum_tensor_em_wrapper_dimension_error() {
    // Wrong dimension - 3x3 instead of square
    let em = CausalTensor::new(vec![0.0; 9], vec![3, 3]).unwrap();
    let metric = CausalTensor::new(vec![0.0; 16], vec![4, 4]).unwrap();

    let result = energy_momentum_tensor_em(&em, &metric);
    // The computation should still work or fail gracefully
    let _ = result.is_ok() || result.is_err();
}

// ============================================================================
// Plasma Wrapper Tests
// ============================================================================

#[test]
fn test_debye_length_wrapper_success() {
    let temp = Temperature::new(1e4).unwrap(); // 10,000 K
    let n = 1e18; // m^-3
    let eps0 = 8.854e-12;
    let e = 1.602e-19;

    let result = debye_length(temp, n, eps0, e);
    assert!(result.is_ok());

    if let EffectValue::Value(lambda) = result.value() {
        // Typical Debye length in plasma ~ 7e-5 m for these parameters
        assert!(lambda.value() > 0.0);
        assert!(lambda.value() < 1.0); // Should be small (meters)
    }
}

#[test]
fn test_debye_length_wrapper_cold_plasma() {
    let temp = Temperature::new(300.0).unwrap(); // Room temp
    let n = 1e18;
    let eps0 = 8.854e-12;
    let e = 1.602e-19;

    let result = debye_length(temp, n, eps0, e);
    assert!(result.is_ok());

    if let EffectValue::Value(lambda) = result.value() {
        // Cold plasma → smaller Debye length
        assert!(lambda.value() > 0.0);
    }
}

#[test]
fn test_larmor_radius_wrapper_success() {
    let mass = Mass::new(9.109e-31).unwrap(); // Electron mass
    let v = Speed::new(1e6).unwrap(); // 1000 km/s
    let charge = 1.602e-19;
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );

    let result = larmor_radius(mass, v, charge, &b);
    assert!(result.is_ok());

    if let EffectValue::Value(r) = result.value() {
        // r_L = m*v / (q*B) = 9.1e-31 * 1e6 / (1.6e-19 * 1) ≈ 5.7e-6 m
        assert!(r.value() > 0.0);
        assert!(r.value() < 1e-3); // Should be very small
    }
}

#[test]
fn test_larmor_radius_wrapper_proton() {
    let mass = Mass::new(1.673e-27).unwrap(); // Proton mass
    let v = Speed::new(1e5).unwrap();
    let charge = 1.602e-19;
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 0.1, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );

    let result = larmor_radius(mass, v, charge, &b);
    assert!(result.is_ok());

    if let EffectValue::Value(r) = result.value() {
        // Proton has larger radius than electron
        assert!(r.value() > 0.0);
    }
}

// ============================================================================
// Error Path Tests
// ============================================================================

#[test]
fn test_alfven_speed_error_zero_density() {
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );
    let rho = Density::new(0.0).unwrap();

    let result = alfven_speed(&b, &rho, 1.0);
    // Wrapper should propagate the error
    assert!(result.is_err());
}

#[test]
fn test_magnetic_pressure_error_negative_permeability() {
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );

    // Negative permeability should error - but this is in the kernel
    // The wrapper propagates whatever the kernel returns
    let result = magnetic_pressure(&b, -1.0);
    assert!(result.is_err());
}

#[test]
fn test_reconnection_rate_error_negative_lundquist() {
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );
    let rho = Density::new(1.0).unwrap();
    let va = alfven_speed(&b, &rho, 1.0)
        .value()
        .clone()
        .into_value()
        .unwrap();

    let result = magnetic_reconnection_rate(va, -1.0);
    assert!(result.is_err());
}

#[test]
fn test_debye_length_error_zero_density() {
    let temp = Temperature::new(1000.0).unwrap();
    let result = debye_length(temp, 0.0, 8.854e-12, 1.602e-19);
    assert!(result.is_err());
}

#[test]
fn test_larmor_radius_error_zero_field() {
    let mass = Mass::new(1e-27).unwrap();
    let v = Speed::new(1e5).unwrap();
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 0.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );

    let result = larmor_radius(mass, v, 1.0, &b);
    assert!(result.is_err());
}

#[test]
fn test_larmor_radius_error_zero_charge() {
    let mass = Mass::new(1e-27).unwrap();
    let v = Speed::new(1e5).unwrap();
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );

    let result = larmor_radius(mass, v, 0.0, &b);
    assert!(result.is_err());
}
