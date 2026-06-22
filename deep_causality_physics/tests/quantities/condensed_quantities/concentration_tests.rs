/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::PhysicsErrorEnum;

#[test]
fn test_concentration_new_valid() {
    let t = deep_causality_tensor::CausalTensor::new(vec![0.1, 0.2, 0.3], vec![3]).unwrap();
    let c = deep_causality_physics::Concentration::new(t.clone());
    assert!(c.is_ok());
    assert_eq!(c.unwrap().inner().shape(), t.shape());
}

#[test]
fn test_concentration_new_negative_rejected() {
    let t = deep_causality_tensor::CausalTensor::new(vec![0.1, -0.5, 0.3], vec![3]).unwrap();
    let c = deep_causality_physics::Concentration::new(t);
    assert!(c.is_err());
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
}
