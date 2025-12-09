/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Frequency, Length, Speed, doppler_effect_kernel, wave_speed_kernel};

// =============================================================================
// wave_speed_kernel Tests
// =============================================================================

#[test]
fn test_wave_speed_kernel_valid() {
    // v = f * λ
    let f = Frequency::new(440.0).unwrap(); // 440 Hz
    let lambda = Length::new(0.775).unwrap(); // ~0.775m for 440Hz in air

    let result = wave_speed_kernel(&f, &lambda);
    assert!(result.is_ok());

    let v = result.unwrap();
    // v = 440 * 0.775 = 341 m/s (approx speed of sound)
    assert!((v.value() - 341.0).abs() < 1.0);
}

#[test]
fn test_wave_speed_kernel_zero_frequency() {
    let f = Frequency::new(0.0).unwrap();
    let lambda = Length::new(1.0).unwrap();

    let result = wave_speed_kernel(&f, &lambda);
    assert!(result.is_ok());

    let v = result.unwrap();
    assert!((v.value() - 0.0).abs() < 1e-10);
}

// =============================================================================
// doppler_effect_kernel Tests
// =============================================================================

#[test]
fn test_doppler_effect_kernel_approaching() {
    // Approaching: f_obs = f_src * (v + vo) / (v - vs)
    let f_src = Frequency::new(1000.0).unwrap();
    let v = Speed::new(340.0).unwrap(); // Speed of sound
    let vo = Speed::new(10.0).unwrap(); // Observer moving towards
    let vs = Speed::new(10.0).unwrap(); // Source moving towards

    let result = doppler_effect_kernel(&f_src, &v, &vo, &vs);
    assert!(result.is_ok());

    let f_obs = result.unwrap();
    // f_obs = 1000 * (340 + 10) / (340 - 10) = 1000 * 350/330 ≈ 1060.6 Hz
    assert!(f_obs.value() > 1000.0, "Observed frequency should increase");
}

#[test]
fn test_doppler_effect_kernel_stationary() {
    // No relative motion
    let f_src = Frequency::new(1000.0).unwrap();
    let v = Speed::new(340.0).unwrap();
    let vo = Speed::new(0.0).unwrap();
    let vs = Speed::new(0.0).unwrap();

    let result = doppler_effect_kernel(&f_src, &v, &vo, &vs);
    assert!(result.is_ok());

    let f_obs = result.unwrap();
    assert!((f_obs.value() - 1000.0).abs() < 1e-10);
}

#[test]
fn test_doppler_effect_kernel_sonic_singularity() {
    // Source speed = wave speed => sonic boom
    let f_src = Frequency::new(1000.0).unwrap();
    let v = Speed::new(340.0).unwrap();
    let vo = Speed::new(0.0).unwrap();
    let vs = Speed::new(340.0).unwrap(); // Source at Mach 1

    let result = doppler_effect_kernel(&f_src, &v, &vo, &vs);
    assert!(result.is_err(), "Sonic singularity should error");
}

#[test]
fn test_doppler_effect_kernel_supersonic_error() {
    // Source speed > wave speed
    let f_src = Frequency::new(1000.0).unwrap();
    let v = Speed::new(340.0).unwrap();
    let vo = Speed::new(0.0).unwrap();
    let vs = Speed::new(400.0).unwrap(); // Supersonic

    let result = doppler_effect_kernel(&f_src, &v, &vo, &vs);
    assert!(result.is_err(), "Supersonic source should error");
}
