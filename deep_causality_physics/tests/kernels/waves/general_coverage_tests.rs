/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage test for the infinity guard in `wave_speed_kernel` (general.rs:25-27).

use deep_causality_physics::{Frequency, Length, PhysicsErrorEnum, wave_speed_kernel};

#[test]
fn test_wave_speed_kernel_infinite_product() {
    // f and λ are each finite but their product overflows to +∞, so the
    // `v.is_infinite()` guard returns NumericalInstability (general.rs:24-28).
    let f = Frequency::<f64>::new(1.0e200).unwrap();
    let lambda = Length::<f64>::new(1.0e200).unwrap();

    let result = wave_speed_kernel(&f, &lambda);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        PhysicsErrorEnum::NumericalInstability(_) => {}
        e => panic!("Expected NumericalInstability, got {e:?}"),
    }
}
