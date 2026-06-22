/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage test for the error arm of the `wave_speed` wrapper (wrappers.rs:22).

use deep_causality_physics::{Frequency, Length, wave_speed};

#[test]
fn test_wave_speed_wrapper_error_path() {
    // f · λ overflows to +∞, driving `wave_speed_kernel` into its infinity
    // guard; the wrapper forwards the error effect (wrappers.rs:22).
    let f = Frequency::<f64>::new(1.0e200).unwrap();
    let lambda = Length::<f64>::new(1.0e200).unwrap();

    let effect = wave_speed(&f, &lambda);
    assert!(!effect.is_ok());
}
