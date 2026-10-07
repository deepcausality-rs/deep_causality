/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::instrument::environment_reading::EnvironmentReading;
use crate::types::instrument::interferometer_configuration::InterferometerConfiguration;
use crate::types::instrument::interferometer_model::InterferometerModel;
use crate::types::instrument::record_fields::entries;
use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use deep_causality_algebra::RealField;
use deep_causality_context::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

/// One data node of an atom interferometer's context.
#[derive(Debug, Clone, PartialEq)]
pub enum InterferometerDatum<R> {
    /// What the instrument is.
    Model(InterferometerModel<R>),
    /// How it is set.
    Configuration(InterferometerConfiguration<R>),
    /// What its surroundings read at one time.
    Environment(EnvironmentReading<R>),
}

impl<R> InterferometerDatum<R> {
    /// The model, when this node holds it.
    pub fn model(&self) -> Option<&InterferometerModel<R>> {
        match self {
            Self::Model(model) => Some(model),
            _ => None,
        }
    }

    /// The configuration, when this node holds it.
    pub fn configuration(&self) -> Option<&InterferometerConfiguration<R>> {
        match self {
            Self::Configuration(configuration) => Some(configuration),
            _ => None,
        }
    }

    /// The environment reading, when this node holds one.
    pub fn environment(&self) -> Option<&EnvironmentReading<R>> {
        match self {
            Self::Environment(reading) => Some(reading),
            _ => None,
        }
    }
}

/// An environment reading of zeros: `Data` requires a default payload.
impl<R: RealField> Default for InterferometerDatum<R> {
    fn default() -> Self {
        Self::Environment(EnvironmentReading::default())
    }
}

/// A `Fields` record of one entry, `model`, `configuration` or `environment`, holding the
/// variant's own record.
impl<R: RealField + Storable + core::fmt::Debug> Storable for InterferometerDatum<R> {
    fn to_record(&self) -> DataRecord {
        let (kind, record) = match self {
            Self::Model(model) => ("model", model.to_record()),
            Self::Configuration(configuration) => ("configuration", configuration.to_record()),
            Self::Environment(reading) => ("environment", reading.to_record()),
        };
        DataRecord::Fields(vec![(kind.to_string(), record)])
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        let mut e = entries(id, record)?;
        let (kind, record) = match (e.pop(), e.is_empty()) {
            (Some(entry), true) => entry,
            _ => {
                return Err(ProjectionError::Rejected(
                    id,
                    "an interferometer datum is one entry".to_string(),
                ));
            }
        };
        match kind.as_str() {
            "model" => InterferometerModel::from_record(id, record).map(Self::Model),
            "configuration" => {
                InterferometerConfiguration::from_record(id, record).map(Self::Configuration)
            }
            "environment" => EnvironmentReading::from_record(id, record).map(Self::Environment),
            other => Err(ProjectionError::Rejected(
                id,
                format!(
                    "an interferometer datum is a model, a configuration or an environment \
                     reading, got '{other}'"
                ),
            )),
        }
    }
}
