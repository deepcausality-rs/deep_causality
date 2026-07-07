/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::kernels::waves::general;
use crate::{Frequency, Length, Speed};
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_core::PropagatingEffect;
use deep_causality_num::FromPrimitive;

/// Monadic wrapper for [`general::wave_speed_kernel`].
pub fn wave_speed<R>(
    frequency: &Frequency<R>,
    wavelength: &Length<R>,
) -> PropagatingEffect<Speed<R>>
where
    R: RealField + Debug,
{
    match general::wave_speed_kernel(frequency, wavelength) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(e.into()),
    }
}

/// Monadic wrapper for [`general::doppler_effect_kernel`] (Approaching case).
pub fn doppler_effect_approaching<R>(
    freq_source: &Frequency<R>,
    wave_speed: &Speed<R>,
    obs_speed: &Speed<R>,
    src_speed: &Speed<R>,
) -> PropagatingEffect<Frequency<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match general::doppler_effect_kernel(freq_source, wave_speed, obs_speed, src_speed) {
        Ok(f) => PropagatingEffect::pure(f),
        Err(e) => PropagatingEffect::from_error(e.into()),
    }
}
