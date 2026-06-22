/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::BandDrudeWeight;

#[test]
fn test_band_drude_weight() {
    let bdw = BandDrudeWeight::<f64>::new(2.5).unwrap();
    assert_eq!(bdw.value(), 2.5);
    let val: f64 = bdw.into();
    assert_eq!(val, 2.5);
}

#[test]
fn test_band_drude_weight_default() {
    let bdw: BandDrudeWeight<f64> = Default::default();
    assert_eq!(bdw.value(), 0.0);
}
