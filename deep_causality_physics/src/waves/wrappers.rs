/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::waves::general;
use crate::{Frequency, Length, Speed};
use deep_causality_core::PropagatingEffect;

/// Monadic wrapper for [`general::wave_speed_kernel`].
///
/// Returns a `PropagatingEffect<Speed>` usable in causal chains.
pub fn wave_speed(frequency: &Frequency, wavelength: &Length) -> PropagatingEffect<Speed> {
    match general::wave_speed_kernel(frequency, wavelength) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(e.into()),
    }
}

/// Monadic wrapper for [`general::doppler_effect_kernel`] (Approaching case).
///
/// Returns a `PropagatingEffect<Frequency>` usable in causal chains.
pub fn doppler_effect_approaching(
    freq_source: &Frequency,
    wave_speed: &Speed,
    obs_speed: &Speed,
    src_speed: &Speed,
) -> PropagatingEffect<Frequency> {
    match general::doppler_effect_kernel(freq_source, wave_speed, obs_speed, src_speed) {
        Ok(f) => PropagatingEffect::pure(f),
        Err(e) => PropagatingEffect::from_error(e.into()),
    }
}
