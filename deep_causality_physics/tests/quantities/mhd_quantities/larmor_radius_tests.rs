/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::LarmorRadius;

#[test]
fn test_larmor_radius() {
    let r = LarmorRadius::<f64>::new(1.0).unwrap();
    assert_eq!(r.value(), 1.0);
    assert!(LarmorRadius::<f64>::new(0.0).is_err()); // Must be positive
    assert!(LarmorRadius::<f64>::new(f64::NAN).is_err());
    assert!(LarmorRadius::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_larmor_radius_new_unchecked() {
    let r = LarmorRadius::<f64>::new_unchecked(1.0);
    assert_eq!(r.value(), 1.0);
}

#[test]
fn test_larmor_radius_default() {
    let r: LarmorRadius<f64> = Default::default();
    assert!(r.value() > 0.0);
}

#[test]
fn test_larmor_radius_new_nan_error() {
    assert!(LarmorRadius::<f64>::new(f64::NAN).is_err());
    assert!(LarmorRadius::<f64>::new(f64::INFINITY).is_err());
}
