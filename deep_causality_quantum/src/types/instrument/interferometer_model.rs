/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::instrument::record_fields::{entries, field, rejected};
use crate::types::pipeline::effective_draws::Fringe;
use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use deep_causality_algebra::RealField;
use deep_causality_context::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

/// What a light-pulse atom interferometer is: its scale factor, its noise and its timing. SI units
/// throughout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterferometerModel<R> {
    k_eff: R,
    interrogation_time: R,
    contrast: R,
    sensitivity: R,
    cycle_time: R,
    white_noise_range: R,
    setup_time: R,
}

impl<R: RealField + core::fmt::Debug> InterferometerModel<R> {
    /// A model from its parameters:
    ///
    /// * `k_eff`, the effective wave number of the two-photon transition, in rad/m;
    /// * `interrogation_time`, `T`, the time between two pulses, in s;
    /// * `contrast`, `C`, the fringe contrast, in `(0, 1]`;
    /// * `sensitivity`, `S`, the acceleration noise density, in (m/s²)/√Hz;
    /// * `cycle_time`, one measurement cycle, in s, at least `2T`, the span of the three pulses;
    /// * `white_noise_range`, the longest averaging time over which the noise stays white, in s;
    /// * `setup_time`, the dead time one configuration change costs, in s.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] naming the first parameter that is not finite;
    /// [`QuantumError::CalculationError`] naming the parameter when `k_eff`, `T`, `S`, the cycle
    /// time or the white-noise range is not positive, the contrast is outside `(0, 1]`, the setup
    /// time is negative, or the cycle is shorter than `2T`.
    pub fn new(
        k_eff: R,
        interrogation_time: R,
        contrast: R,
        sensitivity: R,
        cycle_time: R,
        white_noise_range: R,
        setup_time: R,
    ) -> Result<Self, QuantumError> {
        let named = [
            ("k_eff", k_eff),
            ("interrogation_time", interrogation_time),
            ("contrast", contrast),
            ("sensitivity", sensitivity),
            ("cycle_time", cycle_time),
            ("white_noise_range", white_noise_range),
            ("setup_time", setup_time),
        ];
        if let Some((name, value)) = named.iter().find(|(_, value)| !value.is_finite()) {
            return Err(QuantumError::NonFiniteValue(format!(
                "interferometer model: {name} must be finite, got {value:?}"
            )));
        }
        let positive = [
            ("k_eff", k_eff),
            ("interrogation_time", interrogation_time),
            ("sensitivity", sensitivity),
            ("cycle_time", cycle_time),
            ("white_noise_range", white_noise_range),
        ];
        if let Some((name, value)) = positive.iter().find(|(_, value)| *value <= R::zero()) {
            return Err(QuantumError::CalculationError(format!(
                "interferometer model: {name} must be positive, got {value:?}"
            )));
        }
        if contrast <= R::zero() || contrast > R::one() {
            return Err(QuantumError::CalculationError(format!(
                "interferometer model: contrast must lie in (0, 1], got {contrast:?}"
            )));
        }
        if setup_time < R::zero() {
            return Err(QuantumError::CalculationError(format!(
                "interferometer model: setup_time must be non-negative, got {setup_time:?}"
            )));
        }
        if cycle_time < interrogation_time + interrogation_time {
            return Err(QuantumError::CalculationError(format!(
                "interferometer model: a cycle of {cycle_time:?} s is shorter than the 2T = \
                 {:?} s the three pulses span",
                interrogation_time + interrogation_time
            )));
        }
        Ok(Self {
            k_eff,
            interrogation_time,
            contrast,
            sensitivity,
            cycle_time,
            white_noise_range,
            setup_time,
        })
    }

    /// The effective wave number, in rad/m.
    pub fn k_eff(&self) -> R {
        self.k_eff
    }

    /// `T`, the time between two pulses, in s.
    pub fn interrogation_time(&self) -> R {
        self.interrogation_time
    }

    /// `C`, the fringe contrast.
    pub fn contrast(&self) -> R {
        self.contrast
    }

    /// `S`, the acceleration noise density, in (m/s²)/√Hz.
    pub fn sensitivity(&self) -> R {
        self.sensitivity
    }

    /// One measurement cycle, in s.
    pub fn cycle_time(&self) -> R {
        self.cycle_time
    }

    /// The longest averaging time over which the noise stays white, in s.
    pub fn white_noise_range(&self) -> R {
        self.white_noise_range
    }

    /// The dead time one configuration change costs, in s.
    pub fn setup_time(&self) -> R {
        self.setup_time
    }

    /// The fringe at mid-fringe, where the read-out `½(1 − C cos Δφ)` with `Δφ = k_eff g T²`
    /// moves with acceleration at its steepest: the operating point ½ and the slope
    /// `C · k_eff · T² / 2` per m/s².
    pub fn fringe(&self) -> Fringe<R> {
        let two = R::one() + R::one();
        Fringe::at(
            R::one() / two,
            self.contrast * self.k_eff * self.interrogation_time * self.interrogation_time / two,
        )
    }
}

/// A `Fields` record with one entry per parameter, under the parameter's name. Reading one back
/// applies [`InterferometerModel::new`], so a record whose values break its rules is `Rejected`.
impl<R: RealField + Storable + core::fmt::Debug> Storable for InterferometerModel<R> {
    fn to_record(&self) -> DataRecord {
        DataRecord::Fields(vec![
            ("k_eff".to_string(), self.k_eff.to_record()),
            (
                "interrogation_time".to_string(),
                self.interrogation_time.to_record(),
            ),
            ("contrast".to_string(), self.contrast.to_record()),
            ("sensitivity".to_string(), self.sensitivity.to_record()),
            ("cycle_time".to_string(), self.cycle_time.to_record()),
            (
                "white_noise_range".to_string(),
                self.white_noise_range.to_record(),
            ),
            ("setup_time".to_string(), self.setup_time.to_record()),
        ])
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        let e = entries(id, record)?;
        Self::new(
            field(id, &e, "k_eff")?,
            field(id, &e, "interrogation_time")?,
            field(id, &e, "contrast")?,
            field(id, &e, "sensitivity")?,
            field(id, &e, "cycle_time")?,
            field(id, &e, "white_noise_range")?,
            field(id, &e, "setup_time")?,
        )
        .map_err(|error| rejected(id, error))
    }
}
