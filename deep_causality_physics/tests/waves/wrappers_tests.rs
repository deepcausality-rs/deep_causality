/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Frequency, Length, Speed, doppler_effect_approaching, wave_speed};

#[test]
fn test_wave_speed_wrapper_success() {
    let f = Frequency::new(440.0).unwrap();
    let lambda = Length::new(0.775).unwrap();

    let effect = wave_speed(&f, &lambda);
    assert!(effect.is_ok());

    let v = effect.value().clone().into_value().unwrap();
    assert!((v.value() - 341.0).abs() < 1.0);
}

#[test]
fn test_doppler_effect_approaching_wrapper_success() {
    let f_src = Frequency::new(1000.0).unwrap();
    let v = Speed::new(340.0).unwrap();
    let vo = Speed::new(10.0).unwrap();
    let vs = Speed::new(10.0).unwrap();

    let effect = doppler_effect_approaching(&f_src, &v, &vo, &vs);
    assert!(effect.is_ok());

    let f_obs = effect.value().clone().into_value().unwrap();
    assert!(f_obs.value() > 1000.0);
}

#[test]
fn test_doppler_effect_approaching_wrapper_stationary() {
    let f_src = Frequency::new(1000.0).unwrap();
    let v = Speed::new(340.0).unwrap();
    let vo = Speed::new(0.0).unwrap();
    let vs = Speed::new(0.0).unwrap();

    let effect = doppler_effect_approaching(&f_src, &v, &vo, &vs);
    assert!(effect.is_ok());

    let f_obs = effect.value().clone().into_value().unwrap();
    assert!((f_obs.value() - 1000.0).abs() < 1e-10);
}

#[test]
fn test_doppler_effect_approaching_wrapper_sonic_error() {
    let f_src = Frequency::new(1000.0).unwrap();
    let v = Speed::new(340.0).unwrap();
    let vo = Speed::new(0.0).unwrap();
    let vs = Speed::new(340.0).unwrap(); // Mach 1

    let effect = doppler_effect_approaching(&f_src, &v, &vo, &vs);
    assert!(effect.is_err());
}
