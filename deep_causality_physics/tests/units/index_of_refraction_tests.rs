/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::IndexOfRefraction;

#[test]
fn test_index_of_refraction_new_valid() {
    // Typical value for water ~ 1.33
    let n = IndexOfRefraction::new(1.33);
    assert!(n.is_ok());
    assert!((n.unwrap().value() - 1.33).abs() < 1e-10);
}

#[test]
fn test_index_of_refraction_vacuum() {
    // Vacuum has n = 1.0
    let n = IndexOfRefraction::new(1.0);
    assert!(n.is_ok());
    assert!((n.unwrap().value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_index_of_refraction_glass() {
    // Crown glass ~ 1.52
    let n = IndexOfRefraction::new(1.52);
    assert!(n.is_ok());
}

#[test]
fn test_index_of_refraction_diamond() {
    // Diamond ~ 2.42
    let n = IndexOfRefraction::new(2.42);
    assert!(n.is_ok());
}

#[test]
fn test_index_of_refraction_metamaterial_negative() {
    // Metamaterials can have negative refractive index
    let n = IndexOfRefraction::new(-1.0);
    assert!(n.is_ok()); // Commented in source: "can be negative in metamaterials"
}

#[test]
fn test_index_of_refraction_zero_error() {
    // Zero index is invalid (causes division errors)
    let n = IndexOfRefraction::new(0.0);
    assert!(n.is_err(), "Zero index should error");
}

#[test]
fn test_index_of_refraction_default() {
    let n = IndexOfRefraction::default();
    // Default is 0.0, which is technically invalid per new() but allowed by Default
    assert!((n.value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_index_of_refraction_into_f64() {
    let n = IndexOfRefraction::new(1.5).unwrap();
    let val: f64 = n.into();
    assert!((val - 1.5).abs() < 1e-10);
}
