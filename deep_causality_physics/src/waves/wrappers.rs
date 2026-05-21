/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::waves::general;
use crate::{Frequency, Length, Speed};
use deep_causality_core::PropagatingEffect;

/// Monadic wrapper for [`general::wave_speed_kernel`].
///
/// Returns a `PropagatingEffect<Speed<f64>>` usable in causal chains.
pub fn wave_speed(
    frequency: &Frequency<f64>,
    wavelength: &Length<f64>,
) -> PropagatingEffect<Speed<f64>> {
    match general::wave_speed_kernel(frequency, wavelength) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(e.into()),
    }
}

/// Monadic wrapper for [`general::doppler_effect_kernel`] (Approaching case).
///
/// Returns a `PropagatingEffect<Frequency<f64>>` usable in causal chains.
pub fn doppler_effect_approaching(
    freq_source: &Frequency<f64>,
    wave_speed: &Speed<f64>,
    obs_speed: &Speed<f64>,
    src_speed: &Speed<f64>,
) -> PropagatingEffect<Frequency<f64>> {
    match general::doppler_effect_kernel(freq_source, wave_speed, obs_speed, src_speed) {
        Ok(f) => PropagatingEffect::pure(f),
        Err(e) => PropagatingEffect::from_error(e.into()),
    }
}
