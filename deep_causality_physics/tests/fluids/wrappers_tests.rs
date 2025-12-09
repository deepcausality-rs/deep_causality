/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Density, Length, Pressure, Speed, bernoulli_pressure, hydrostatic_pressure,
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
