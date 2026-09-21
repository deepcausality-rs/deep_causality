/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::PhysicsErrorEnum;

#[test]
fn test_concentration_new_valid() {
    let t = deep_causality_tensor::CausalTensor::new(vec![0.1, 0.2, 0.3], vec![3]).unwrap();
    let c = deep_causality_physics::Concentration::new(t.clone()).unwrap();
    // Shape alone would pass for a constructor that dropped or zeroed the data.
    assert_eq!(c.inner().shape(), t.shape());
    assert_eq!(c.inner().as_slice(), t.as_slice());
}

#[test]
fn test_concentration_new_negative_rejected() {
    let t = deep_causality_tensor::CausalTensor::new(vec![0.1, -0.5, 0.3], vec![3]).unwrap();
    let c = deep_causality_physics::Concentration::new(t);
    match c.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(_) => {}
        other => panic!("expected PhysicalInvariantBroken, got {other:?}"),
    }
}

#[test]
fn test_concentration_new_unchecked() {
    // new_unchecked bypasses the non-negativity check.
    let t = deep_causality_tensor::CausalTensor::new(vec![-1.0, 0.0, 2.0], vec![3]).unwrap();
    let c = deep_causality_physics::Concentration::new_unchecked(t.clone());
    assert_eq!(c.inner().shape(), t.shape());
    // The negative entry must survive verbatim; that is what "unchecked" means here.
    assert_eq!(c.inner().as_slice(), t.as_slice());
    assert_eq!(c.inner().as_slice()[0], -1.0);
}
