/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use alloc::format;
use deep_causality_algebra::RealField;

/// What one shot costs on the instrument, and how long it may integrate: the time per shot, in
/// seconds, and the white-noise range, the longest averaging time over which the noise stays
/// white, in seconds. A separation that needs more shots than the range holds cannot be bought
/// with time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstrumentTime<R> {
    shot_time: R,
    white_noise_range: R,
}

impl<R: RealField + core::fmt::Debug> InstrumentTime<R> {
    /// The instrument time of `shot_time` seconds per shot within `white_noise_range` seconds.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] when either is not finite;
    /// [`QuantumError::CalculationError`] when the shot time is not positive or the range is
    /// shorter than one shot.
    pub fn new(shot_time: R, white_noise_range: R) -> Result<Self, QuantumError> {
        if !shot_time.is_finite() || !white_noise_range.is_finite() {
            return Err(QuantumError::NonFiniteValue(format!(
                "instrument time needs a finite shot time and white-noise range, got \
                 {shot_time:?} and {white_noise_range:?}"
            )));
        }
        if shot_time <= R::zero() {
            return Err(QuantumError::CalculationError(format!(
                "a shot takes positive time, got {shot_time:?}"
            )));
        }
        if white_noise_range < shot_time {
            return Err(QuantumError::CalculationError(format!(
                "a white-noise range of {white_noise_range:?} s holds no shot of {shot_time:?} s"
            )));
        }
        Ok(Self {
            shot_time,
            white_noise_range,
        })
    }
}

impl<R: RealField> InstrumentTime<R> {
    /// The time per shot, in seconds.
    pub fn shot_time(&self) -> R {
        self.shot_time
    }

    /// The white-noise range, in seconds.
    pub fn white_noise_range(&self) -> R {
        self.white_noise_range
    }

    /// The most shots one experiment may integrate, `⌊range / shot time⌋`, and `u64::MAX` past
    /// what a `u64` holds.
    pub fn max_shots(&self) -> u64 {
        (self.white_noise_range / self.shot_time)
            .floor()
            .to_u64()
            .unwrap_or(u64::MAX)
    }
}
