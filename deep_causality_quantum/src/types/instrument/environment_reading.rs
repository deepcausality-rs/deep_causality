/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::instrument::record_fields::{entries, field, rejected};
use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use deep_causality_algebra::RealField;
use deep_causality_context::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

/// What the instrument's surroundings read at one time: the Earth tide's acceleration along the
/// sensitive axis (m/s²), the ambient temperature (K) and the ambient magnetic field (T). The
/// time is the `DiscreteTime` node the context keeps beside the reading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvironmentReading<R> {
    earth_tide: R,
    temperature: R,
    field: R,
}

impl<R: RealField + core::fmt::Debug> EnvironmentReading<R> {
    /// A reading of `earth_tide` (m/s²), `temperature` (K) and `field` (T).
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] naming the first value that is not finite.
    pub fn new(earth_tide: R, temperature: R, field: R) -> Result<Self, QuantumError> {
        let named = [
            ("earth_tide", earth_tide),
            ("temperature", temperature),
            ("field", field),
        ];
        if let Some((name, value)) = named.iter().find(|(_, value)| !value.is_finite()) {
            return Err(QuantumError::NonFiniteValue(format!(
                "environment reading: {name} must be finite, got {value:?}"
            )));
        }
        Ok(Self {
            earth_tide,
            temperature,
            field,
        })
    }

    /// The Earth tide's acceleration along the sensitive axis, in m/s².
    pub fn earth_tide(&self) -> R {
        self.earth_tide
    }

    /// The ambient temperature, in K.
    pub fn temperature(&self) -> R {
        self.temperature
    }

    /// The ambient magnetic field, in T.
    pub fn field(&self) -> R {
        self.field
    }
}

/// A reading of zeros.
impl<R: RealField> Default for EnvironmentReading<R> {
    fn default() -> Self {
        Self {
            earth_tide: R::zero(),
            temperature: R::zero(),
            field: R::zero(),
        }
    }
}

/// A `Fields` record with one entry per value, under the value's name.
impl<R: RealField + Storable + core::fmt::Debug> Storable for EnvironmentReading<R> {
    fn to_record(&self) -> DataRecord {
        DataRecord::Fields(vec![
            ("earth_tide".to_string(), self.earth_tide.to_record()),
            ("temperature".to_string(), self.temperature.to_record()),
            ("field".to_string(), self.field.to_record()),
        ])
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        let e = entries(id, record)?;
        Self::new(
            field(id, &e, "earth_tide")?,
            field(id, &e, "temperature")?,
            field(id, &e, "field")?,
        )
        .map_err(|error| rejected(id, error))
    }
}
