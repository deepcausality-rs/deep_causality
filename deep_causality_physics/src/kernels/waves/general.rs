/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Wave Mechanics
//!
//! This module implements core wave mechanics kernels, focusing on classical wave propagation
//! and Doppler effects. The implementation prioritizes type safety and physical correctness,
//! ensuring that operations respect domain limits (e.g., sonic singularities).
use crate::{Frequency, Length, PhysicsError, Speed};
use deep_causality_num::{FromPrimitive, RealField};

/// Calculates the speed of a wave given its frequency and wavelength.
pub fn wave_speed_kernel<R>(
    frequency: &Frequency<R>,
    wavelength: &Length<R>,
) -> Result<Speed<R>, PhysicsError>
where
    R: RealField,
{
    let v = frequency.value() * wavelength.value();

    if v.is_infinite() {
        return Err(PhysicsError::NumericalInstability(
            "Wave speed calculation resulted in infinity".into(),
        ));
    }

    Speed::new(v)
}

/// Calculates the observed frequency due to the Doppler effect for longitudinal motion.
///
/// Formula: $f_{obs} = f_{src} \frac{v + v_o}{v - v_s}$
pub fn doppler_effect_kernel<R>(
    freq_source: &Frequency<R>,
    wave_speed: &Speed<R>,
    obs_speed: &Speed<R>,
    src_speed: &Speed<R>,
) -> Result<Frequency<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let v = wave_speed.value();
    let vo = obs_speed.value();
    let vs = src_speed.value();

    let denominator = v - vs;

    let eps = R::from_f64(1e-9)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1e-9)".into()))?;
    if denominator <= eps {
        return Err(PhysicsError::MetricSingularity(
            "Source speed equals or exceeds wave speed - Sonic Singularity".into(),
        ));
    }

    let f_obs = freq_source.value() * ((v + vo) / denominator);

    Frequency::new(f_obs)
}
