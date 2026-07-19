/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::MassFlowRate;

#[test]
fn test_mass_flow_rate_valid() {
    let m = MassFlowRate::<f64>::new(3.5).unwrap();
    assert_eq!(m.value(), 3.5);
    let zero = MassFlowRate::<f64>::new(0.0).unwrap();
    assert_eq!(zero.value(), 0.0);
}

#[test]
fn test_mass_flow_rate_rejects_negative() {
    assert!(MassFlowRate::<f64>::new(-1.0).is_err());
}

#[test]
fn test_mass_flow_rate_rejects_nonfinite() {
    assert!(MassFlowRate::<f64>::new(f64::NAN).is_err());
    assert!(MassFlowRate::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_mass_flow_rate_new_unchecked() {
    let m = MassFlowRate::<f64>::new_unchecked(2.0);
    assert_eq!(m.value(), 2.0);
}

#[test]
fn test_mass_flow_rate_default() {
    let m: MassFlowRate<f64> = Default::default();
    assert_eq!(m.value(), 0.0);
}

#[test]
fn test_mass_flow_rate_into_f64() {
    let m = MassFlowRate::<f64>::new(4.25).unwrap();
    let v: f64 = m.into();
    assert_eq!(v, 4.25);
}
