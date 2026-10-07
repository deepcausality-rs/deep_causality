/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::instrument::record_fields::{entries, field, rejected};
use crate::types::instrument::wave_vector::WaveVector;
use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_context::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

/// How an atom interferometer is set: the knobs an experiment turns. SI units throughout; angles
/// in radians, the cloud position in metres in the instrument frame (x and y horizontal, z up).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterferometerConfiguration<R> {
    wave_vector: WaveVector,
    heading: R,
    tilt_offset: R,
    bias_field: R,
    atom_temperature: R,
    rabi_frequency: R,
    cloud_position: [R; 3],
    accelerometer_correction: bool,
}

impl<R: RealField + core::fmt::Debug> InterferometerConfiguration<R> {
    /// A configuration at `bias_field` (T), `atom_temperature` (K) and `rabi_frequency` (rad/s),
    /// with the wave vector up, heading and tilt offset zero, the cloud at the origin and the
    /// accelerometer correction off. The `with_` methods set the rest.
    ///
    /// # Errors
    ///
    /// As the `with_` method of each of the three.
    pub fn new(
        bias_field: R,
        atom_temperature: R,
        rabi_frequency: R,
    ) -> Result<Self, QuantumError> {
        Self {
            wave_vector: WaveVector::Up,
            heading: R::zero(),
            tilt_offset: R::zero(),
            bias_field: R::zero(),
            atom_temperature: R::zero(),
            rabi_frequency: R::one(),
            cloud_position: [R::zero(); 3],
            accelerometer_correction: false,
        }
        .with_bias_field(bias_field)?
        .with_atom_temperature(atom_temperature)?
        .with_rabi_frequency(rabi_frequency)
    }

    /// The same configuration with the wave vector pointing `wave_vector`.
    pub fn with_wave_vector(mut self, wave_vector: WaveVector) -> Self {
        self.wave_vector = wave_vector;
        self
    }

    /// The same configuration at `heading` (rad).
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] when `heading` is not finite.
    pub fn with_heading(mut self, heading: R) -> Result<Self, QuantumError> {
        self.heading = finite("heading", heading)?;
        Ok(self)
    }

    /// The same configuration at a tilt offset from vertical of `tilt_offset` (rad).
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] when `tilt_offset` is not finite.
    pub fn with_tilt_offset(mut self, tilt_offset: R) -> Result<Self, QuantumError> {
        self.tilt_offset = finite("tilt_offset", tilt_offset)?;
        Ok(self)
    }

    /// The same configuration at a bias field of `bias_field` (T).
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] when `bias_field` is not finite.
    pub fn with_bias_field(mut self, bias_field: R) -> Result<Self, QuantumError> {
        self.bias_field = finite("bias_field", bias_field)?;
        Ok(self)
    }

    /// The same configuration at an atom temperature of `atom_temperature` (K). Zero is accepted,
    /// as the limit a temperature scan extrapolates to.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] when it is not finite; [`QuantumError::CalculationError`]
    /// when it is negative.
    pub fn with_atom_temperature(mut self, atom_temperature: R) -> Result<Self, QuantumError> {
        if finite("atom_temperature", atom_temperature)? < R::zero() {
            return Err(QuantumError::CalculationError(format!(
                "interferometer configuration: atom_temperature must be non-negative, got \
                 {atom_temperature:?}"
            )));
        }
        self.atom_temperature = atom_temperature;
        Ok(self)
    }

    /// The same configuration at a Rabi frequency of `rabi_frequency` (rad/s).
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] when it is not finite; [`QuantumError::CalculationError`]
    /// when it is not positive.
    pub fn with_rabi_frequency(mut self, rabi_frequency: R) -> Result<Self, QuantumError> {
        if finite("rabi_frequency", rabi_frequency)? <= R::zero() {
            return Err(QuantumError::CalculationError(format!(
                "interferometer configuration: rabi_frequency must be positive, got \
                 {rabi_frequency:?}"
            )));
        }
        self.rabi_frequency = rabi_frequency;
        Ok(self)
    }

    /// The same configuration with the cloud starting at `cloud_position` (m).
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] when a coordinate is not finite.
    pub fn with_cloud_position(mut self, cloud_position: [R; 3]) -> Result<Self, QuantumError> {
        for coordinate in cloud_position {
            finite("cloud_position", coordinate)?;
        }
        self.cloud_position = cloud_position;
        Ok(self)
    }

    /// The same configuration with the accelerometer correction applied or not.
    pub fn with_accelerometer_correction(mut self, applied: bool) -> Self {
        self.accelerometer_correction = applied;
        self
    }

    /// The direction of the effective wave vector.
    pub fn wave_vector(&self) -> WaveVector {
        self.wave_vector
    }

    /// The heading, in rad.
    pub fn heading(&self) -> R {
        self.heading
    }

    /// The tilt offset from vertical, in rad.
    pub fn tilt_offset(&self) -> R {
        self.tilt_offset
    }

    /// The bias field, in T.
    pub fn bias_field(&self) -> R {
        self.bias_field
    }

    /// The atom temperature, in K.
    pub fn atom_temperature(&self) -> R {
        self.atom_temperature
    }

    /// The Rabi frequency, in rad/s.
    pub fn rabi_frequency(&self) -> R {
        self.rabi_frequency
    }

    /// The cloud's starting position, in m.
    pub fn cloud_position(&self) -> [R; 3] {
        self.cloud_position
    }

    /// Whether the accelerometer correction is applied.
    pub fn accelerometer_correction(&self) -> bool {
        self.accelerometer_correction
    }
}

fn finite<R: RealField + core::fmt::Debug>(name: &str, value: R) -> Result<R, QuantumError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(QuantumError::NonFiniteValue(format!(
            "interferometer configuration: {name} must be finite, got {value:?}"
        )))
    }
}

/// A `Fields` record with one entry per knob, under the knob's name; the cloud position is a
/// `List` of three. Reading one back applies the `with_` methods, so a record whose values break
/// their rules is `Rejected`.
impl<R: RealField + Storable + core::fmt::Debug> Storable for InterferometerConfiguration<R> {
    fn to_record(&self) -> DataRecord {
        DataRecord::Fields(vec![
            ("wave_vector".to_string(), self.wave_vector.to_record()),
            ("heading".to_string(), self.heading.to_record()),
            ("tilt_offset".to_string(), self.tilt_offset.to_record()),
            ("bias_field".to_string(), self.bias_field.to_record()),
            (
                "atom_temperature".to_string(),
                self.atom_temperature.to_record(),
            ),
            (
                "rabi_frequency".to_string(),
                self.rabi_frequency.to_record(),
            ),
            (
                "cloud_position".to_string(),
                self.cloud_position.to_vec().to_record(),
            ),
            (
                "accelerometer_correction".to_string(),
                self.accelerometer_correction.to_record(),
            ),
        ])
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        let e = entries(id, record)?;
        let wave_vector: WaveVector = field(id, &e, "wave_vector")?;
        let heading: R = field(id, &e, "heading")?;
        let tilt_offset: R = field(id, &e, "tilt_offset")?;
        let bias_field: R = field(id, &e, "bias_field")?;
        let atom_temperature: R = field(id, &e, "atom_temperature")?;
        let rabi_frequency: R = field(id, &e, "rabi_frequency")?;
        let position: Vec<R> = field(id, &e, "cloud_position")?;
        let accelerometer_correction: bool = field(id, &e, "accelerometer_correction")?;
        let cloud_position: [R; 3] = position.try_into().map_err(|p: Vec<R>| {
            ProjectionError::Rejected(
                id,
                format!("a cloud position has 3 coordinates, got {}", p.len()),
            )
        })?;
        Self::new(bias_field, atom_temperature, rabi_frequency)
            .and_then(|c| c.with_heading(heading))
            .and_then(|c| c.with_tilt_offset(tilt_offset))
            .and_then(|c| c.with_cloud_position(cloud_position))
            .map(|c| {
                c.with_wave_vector(wave_vector)
                    .with_accelerometer_correction(accelerometer_correction)
            })
            .map_err(|error| rejected(id, error))
    }
}
