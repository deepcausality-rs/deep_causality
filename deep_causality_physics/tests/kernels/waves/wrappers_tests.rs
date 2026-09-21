/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Frequency, Length, Speed, doppler_effect_approaching, doppler_effect_kernel, wave_speed,
    wave_speed_kernel,
};

#[test]
fn test_wave_speed_wrapper_success() {
    let f = Frequency::<f64>::new(440.0).unwrap();
    let lambda = Length::<f64>::new(0.775).unwrap();

    let effect = wave_speed(&f, &lambda);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        wave_speed_kernel(&f, &lambda).unwrap(),
        "wave_speed must carry the value its kernel produced"
    );

    let v = effect.value_cloned().unwrap();
    assert!((v.value() - 341.0).abs() < 1.0);
}

#[test]
fn test_doppler_effect_approaching_wrapper_success() {
    let f_src = Frequency::<f64>::new(1000.0).unwrap();
    let v = Speed::<f64>::new(340.0).unwrap();
    let vo = Speed::<f64>::new(10.0).unwrap();
    let vs = Speed::<f64>::new(10.0).unwrap();

    // Delegation, not merely success. `wave_speed` in this file already had this assertion;
    // `doppler_effect_approaching` did not.
    let effect = doppler_effect_approaching(&f_src, &v, &vo, &vs);
    assert_eq!(
        effect.value_cloned().unwrap(),
        doppler_effect_kernel(&f_src, &v, &vo, &vs).unwrap(),
        "doppler_effect_approaching must carry the value its kernel produced"
    );

    // 1000 * (340 + 10) / (340 - 10) = 1000 * 350/330.
    let f_obs = effect.value_cloned().unwrap();
    assert!(
        (f_obs.value() - 1_000.0 * 350.0 / 330.0).abs() < 1e-9,
        "f_obs = {}",
        f_obs.value()
    );
}

#[test]
fn test_doppler_effect_approaching_wrapper_stationary() {
    let f_src = Frequency::<f64>::new(1000.0).unwrap();
    let v = Speed::<f64>::new(340.0).unwrap();
    let vo = Speed::<f64>::new(0.0).unwrap();
    let vs = Speed::<f64>::new(0.0).unwrap();

    let effect = doppler_effect_approaching(&f_src, &v, &vo, &vs);
    assert!(effect.is_ok());

    let f_obs = effect.value_cloned().unwrap();
    assert!((f_obs.value() - 1000.0).abs() < 1e-10);
}

#[test]
fn test_doppler_effect_approaching_wrapper_sonic_error() {
    let f_src = Frequency::<f64>::new(1000.0).unwrap();
    let v = Speed::<f64>::new(340.0).unwrap();
    let vo = Speed::<f64>::new(0.0).unwrap();
    let vs = Speed::<f64>::new(340.0).unwrap(); // Mach 1

    let effect = doppler_effect_approaching(&f_src, &v, &vo, &vs);
    assert!(effect.is_err());
}
