/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{AmountOfSubstance, PhysicsErrorEnum};

#[test]
fn test_amount_of_substance_new_valid() {
    let amount = AmountOfSubstance::<f64>::new(1.0);
    assert!(amount.is_ok());
    assert!((amount.unwrap().value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_amount_of_substance_new_negative_error() {
    let amount = AmountOfSubstance::<f64>::new(-1.0);
    assert!(amount.is_err());
    match &amount.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("AmountOfSubstance") || msg.contains("Negative"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_amount_of_substance_new_unchecked() {
    let amount = AmountOfSubstance::<f64>::new_unchecked(5.0);
    assert!((amount.value() - 5.0).abs() < 1e-10);
}

#[test]
fn test_amount_of_substance_into_f64() {
    let amount = AmountOfSubstance::<f64>::new(2.5).unwrap();
    let val: f64 = amount.into();
    assert!((val - 2.5).abs() < 1e-10);
}

#[test]
fn test_amount_of_substance_default() {
    let a: AmountOfSubstance<f64> = AmountOfSubstance::default();
    assert_eq!(a.value(), 0.0);
}
