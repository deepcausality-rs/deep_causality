/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage for the propulsion constants and their typed real-field accessors.

use deep_causality_physics::{
    JARVINEN_ADAMS_TRANSITION_CT_M2, JARVINEN_ADAMS_TRANSITION_PRESSURE_RATIO_LO,
    KEYES_HEFNER_PERIPHERAL_RIPPLE_CT, SIBULKIN_SCALING_COEFFICIENT,
    jarvinen_adams_transition_ct_m2, jarvinen_adams_transition_pressure_ratio_lo,
    sibulkin_scaling_coefficient,
};

#[test]
fn test_transition_ct_accessor_matches_constant() {
    let v: f64 = jarvinen_adams_transition_ct_m2();
    assert_eq!(v, JARVINEN_ADAMS_TRANSITION_CT_M2);
    assert_eq!(v, 1.0);
}

#[test]
fn test_transition_pressure_ratio_accessor_matches_constant() {
    let v: f64 = jarvinen_adams_transition_pressure_ratio_lo();
    assert_eq!(v, JARVINEN_ADAMS_TRANSITION_PRESSURE_RATIO_LO);
    assert_eq!(v, 7.0);
}

#[test]
fn test_sibulkin_accessor_matches_constant() {
    let v: f64 = sibulkin_scaling_coefficient();
    assert_eq!(v, SIBULKIN_SCALING_COEFFICIENT);
    assert_eq!(v, 0.4);
}

#[test]
fn test_peripheral_ripple_onset_value() {
    // Keyes-Hefner peripheral bow-shock rippling onset (Korzun survey p. 6).
    assert_eq!(KEYES_HEFNER_PERIPHERAL_RIPPLE_CT, 3.0);
}
