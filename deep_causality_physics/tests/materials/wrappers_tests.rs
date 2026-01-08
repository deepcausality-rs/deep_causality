/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Temperature, hookes_law, thermal_expansion, von_mises_stress};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// hookes_law Wrapper Tests
// =============================================================================

#[test]
fn test_hookes_law_wrapper_success() {
    let stiffness = CausalTensor::new(vec![0.0; 81], vec![3, 3, 3, 3]).unwrap();
    let strain = CausalTensor::new(vec![0.0; 9], vec![3, 3]).unwrap();

    let effect = hookes_law(&stiffness, &strain);
    assert!(effect.is_ok());
}

#[test]
fn test_hookes_law_wrapper_error() {
    let stiffness = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap(); // Wrong rank
    let strain = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();

    let effect = hookes_law(&stiffness, &strain);
    assert!(effect.is_err());
}

// =============================================================================
// von_mises_stress Wrapper Tests
// =============================================================================

#[test]
fn test_von_mises_stress_wrapper_success() {
    let stress = CausalTensor::new(
        vec![100e6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vec![3, 3],
    )
    .unwrap();

    let effect = von_mises_stress(&stress);
    assert!(effect.is_ok());
}

#[test]
fn test_von_mises_stress_wrapper_error() {
    let stress = CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap(); // Wrong shape

    let effect = von_mises_stress(&stress);
    assert!(effect.is_err());
}

// =============================================================================
// thermal_expansion Wrapper Tests
// =============================================================================

#[test]
fn test_thermal_expansion_wrapper_success() {
    let alpha = 12e-6;
    let delta_temp = Temperature::new(50.0).unwrap();

    let effect = thermal_expansion(alpha, delta_temp);
    assert!(effect.is_ok());
}
