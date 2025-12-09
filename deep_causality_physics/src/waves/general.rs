/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Wave Mechanics
//!
//! This module implements core wave mechanics kernels, focusing on classical wave propagation
//! and Doppler effects. The implementation prioritizes type safety and physical correctness,
//! ensuring that operations respect domain limits (e.g., sonic singularities).
//!
//! ## Mathematical Context
//!
//! - **Wave Speed (`v`)**: The speed at which the wave phase propagates.
//!   Formula: $v = f \cdot \lambda$
//!
//! - **Doppler Effect**: Calculating the observed frequency $f_{obs}$ based on the relative
//!   velocities of the source ($v_s$) and observer ($v_o$) relative to the medium.
//!   Formula (General Longitudinal):
//!   $$ f_{obs} = f_{src} \left( \frac{v \pm v_o}{v \mp v_s} \right) $$
//!   where signs depend on direction (approaching vs receding).
use crate::{Frequency, Length, PhysicsError, PhysicsErrorEnum, Speed};
use alloc::format;
use deep_causality_core::PropagatingEffect;

// ============================================================================
// Kernels (Pure Logic)
// ============================================================================

/// Calculates the speed of a wave given its frequency and wavelength.
///
/// # Arguments
/// * `frequency` - The frequency ($f$) of the wave.
/// * `wavelength` - The wavelength ($\lambda$) of the wave.
///
/// # Returns
/// * `Ok(Speed)` - The calculated wave speed.
/// * `Err(PhysicsError)` - If the resulting speed violates physical invariants (negative), though types prevent this input.
pub fn wave_speed_kernel(
    frequency: &Frequency,
    wavelength: &Length,
) -> Result<Speed, PhysicsError> {
    // v = f * lambda
    let v = frequency.value() * wavelength.value();

    // Check for potential overflow or infinite values if required, though basic mul is usually safe.
    if v.is_infinite() {
        return Err(PhysicsError::new(PhysicsErrorEnum::NumericalInstability(
            "Wave speed calculation resulted in infinity".into(),
        )));
    }

    Speed::new(v)
}

/// Calculates the observed frequency due to the Doppler effect for longitudinal motion.
///
/// This kernel assumes the "Approaching" scenario where source and observer move towards each other.
/// For receding motion, one might intuitively negate the speeds, but since `Speed` is non-negative,
/// a separate function or directional flag would be safer. This implementation strictly valid
/// for the **Approaching** case:
/// - Observer moving towards source (+vo)
/// - Source moving towards observer (-vs in denominator)
///
/// Formula: $f_{obs} = f_{src} \frac{v + v_o}{v - v_s}$
///
/// # Arguments
/// * `freq_source` - Frequency emitted by the source.
/// * `wave_speed` - Speed of the wave in the medium ($v$).
/// * `obs_speed` - Speed of the observer relative to the medium ($v_o$).
/// * `src_speed` - Speed of the source relative to the medium ($v_s$).
///
/// # Errors
/// * `PhysicsError::MetricSingularity` - If $v_s \ge v$, causing a sonic boom (denominator zero or negative).
pub fn doppler_effect_kernel(
    freq_source: &Frequency,
    wave_speed: &Speed,
    obs_speed: &Speed, // Observer moving towards source
    src_speed: &Speed, // Source moving towards observer
) -> Result<Frequency, PhysicsError> {
    let v = wave_speed.value();
    let vo = obs_speed.value();
    let vs = src_speed.value();

    // Check for sonic boom / singularity (Source catching up to wavefronts)
    let denominator = v - vs;

    // Use an epsilon for float comparison to catch effective zeros
    if denominator <= 1e-9 {
        return Err(PhysicsError::new(PhysicsErrorEnum::MetricSingularity(
            format!(
                "Source speed ({}) equals or exceeds wave speed ({}) - Sonic Singularity",
                vs, v
            ),
        )));
    }

    // Calculate observed frequency
    // f_obs = f_src * (v + vo) / (v - vs)
    let f_obs = freq_source.value() * ((v + vo) / denominator);

    // Construct result, validating constraints
    Frequency::new(f_obs)
}

// ============================================================================
// Wrappers (Monadic Composition)
// ============================================================================

/// Monadic wrapper for `wave_speed_kernel`.
///
/// Returns a `PropagatingEffect<Speed>` usable in causal chains.
pub fn wave_speed(frequency: &Frequency, wavelength: &Length) -> PropagatingEffect<Speed> {
    match wave_speed_kernel(frequency, wavelength) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(e.into()),
    }
}

/// Monadic wrapper for `doppler_effect_kernel` (Approaching case).
///
/// Returns a `PropagatingEffect<Frequency>` usable in causal chains.
pub fn doppler_effect_approaching(
    freq_source: &Frequency,
    wave_speed: &Speed,
    obs_speed: &Speed,
    src_speed: &Speed,
) -> PropagatingEffect<Frequency> {
    match doppler_effect_kernel(freq_source, wave_speed, obs_speed, src_speed) {
        Ok(f) => PropagatingEffect::pure(f),
        Err(e) => PropagatingEffect::from_error(e.into()),
    }
}
