/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{ElectricPotential, MagneticFlux, PhysicalField};

// =============================================================================
// ElectricPotential Tests
// =============================================================================

#[test]
fn test_electric_potential_new_valid() {
    let pot = ElectricPotential::<f64>::new(120.0);
    assert!(pot.is_ok());
    assert!((pot.unwrap().value() - 120.0).abs() < 1e-10);
}

#[test]
fn test_electric_potential_negative() {
    // Potential can be negative
    let pot = ElectricPotential::<f64>::new(-5.0);
    assert!(pot.is_ok());
    assert!((pot.unwrap().value() - (-5.0)).abs() < 1e-10);
}

#[test]
fn test_electric_potential_new_unchecked() {
    let pot = ElectricPotential::<f64>::new_unchecked(42.0);
    assert!((pot.value() - 42.0).abs() < 1e-10);
}

#[test]
fn test_electric_potential_into_f64() {
    let pot = ElectricPotential::<f64>::new(10.0).unwrap();
    let val: f64 = pot.into();
    assert!((val - 10.0).abs() < 1e-10);
}

#[test]
fn test_electric_potential_default() {
    let pot = ElectricPotential::<f64>::default();
    assert!((pot.value() - 0.0).abs() < 1e-10);
}

// =============================================================================
// MagneticFlux Tests
// =============================================================================

#[test]
fn test_magnetic_flux_new_valid() {
    let flux = MagneticFlux::<f64>::new(0.5);
    assert!(flux.is_ok());
    assert!((flux.unwrap().value() - 0.5).abs() < 1e-10);
}

#[test]
fn test_magnetic_flux_negative() {
    // Flux can be negative (direction dependent)
    let flux = MagneticFlux::<f64>::new(-1.0);
    assert!(flux.is_ok());
}

#[test]
fn test_magnetic_flux_new_unchecked() {
    let flux = MagneticFlux::<f64>::new_unchecked(2.5);
    assert!((flux.value() - 2.5).abs() < 1e-10);
}

#[test]
fn test_magnetic_flux_into_f64() {
    let flux = MagneticFlux::<f64>::new(3.0).unwrap();
    let val: f64 = flux.into();
    assert!((val - 3.0).abs() < 1e-10);
}

// =============================================================================
// PhysicalField Tests
// =============================================================================

#[test]
fn test_physical_field_default() {
    let field = PhysicalField::<f64>::default();
    assert!((field.inner().data()[0] - 0.0).abs() < 1e-10);
}

#[test]
fn test_physical_field_new_and_accessors() {
    let mv = CausalMultiVector::new(
        vec![1.0, 2.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let field = PhysicalField::<f64>::new(mv.clone());

    assert_eq!(field.inner().data(), mv.data());

    let inner = field.into_inner();
    assert_eq!(inner.data(), mv.data());
}

#[test]
fn test_magnetic_flux_default() {
    // em/mod.rs:43-45
    let flux = MagneticFlux::<f64>::default();
    assert!((flux.value() - 0.0).abs() < 1e-10);
}
