/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Density, NozzleExitState, Pressure, Speed, Temperature};

fn sample() -> NozzleExitState<f64> {
    NozzleExitState::new(
        2.94,
        Pressure::new(60_000.0).unwrap(),
        Temperature::new(1100.0).unwrap(),
        Density::new(0.19).unwrap(),
        Speed::new(1950.0).unwrap(),
    )
}

#[test]
fn test_nozzle_exit_state_getters() {
    let s = sample();
    assert_eq!(s.mach(), 2.94);
    assert_eq!(s.pressure().value(), 60_000.0);
    assert_eq!(s.temperature().value(), 1100.0);
    assert_eq!(s.density().value(), 0.19);
    assert_eq!(s.velocity().value(), 1950.0);
}

#[test]
fn test_nozzle_exit_state_default() {
    let s: NozzleExitState<f64> = Default::default();
    assert_eq!(s.mach(), 0.0);
    assert_eq!(s.pressure().value(), 0.0);
    assert_eq!(s.temperature().value(), 0.0);
    assert_eq!(s.density().value(), 0.0);
    assert_eq!(s.velocity().value(), 0.0);
}

#[test]
fn test_nozzle_exit_state_clone_eq() {
    let s = sample();
    let t = s;
    assert_eq!(s, t);
}
