/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

impl Storable for f64 {
    fn to_record(&self) -> DataRecord {
        DataRecord::Number(*self)
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match record {
            DataRecord::Number(value) => Ok(value),
            other => Err(ProjectionError::WrongPayload(
                id,
                "Number",
                other.kind_name(),
            )),
        }
    }
}

/// Widened into a `Number` exactly, so every value an `f32` writes reads back exactly. Reading a
/// `Number` rounds it to the nearest `f32`: a magnitude too small for the smallest `f32` subnormal
/// becomes a zero of the same sign, and a finite double too large for a finite `f32` is refused as
/// `Scalar`. An infinite or `NaN` double is what it was.
impl Storable for f32 {
    fn to_record(&self) -> DataRecord {
        DataRecord::Number(f64::from(*self))
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match record {
            DataRecord::Number(value) => {
                let narrowed = value as f32;
                if value.is_finite() && !narrowed.is_finite() {
                    Err(ProjectionError::Scalar(id, value))
                } else {
                    Ok(narrowed)
                }
            }
            other => Err(ProjectionError::WrongPayload(
                id,
                "Number",
                other.kind_name(),
            )),
        }
    }
}

impl Storable for u64 {
    fn to_record(&self) -> DataRecord {
        DataRecord::Count(*self)
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match record {
            DataRecord::Count(value) => Ok(value),
            other => Err(ProjectionError::WrongPayload(
                id,
                "Count",
                other.kind_name(),
            )),
        }
    }
}

impl Storable for i64 {
    fn to_record(&self) -> DataRecord {
        DataRecord::Integer(*self)
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match record {
            DataRecord::Integer(value) => Ok(value),
            other => Err(ProjectionError::WrongPayload(
                id,
                "Integer",
                other.kind_name(),
            )),
        }
    }
}

impl Storable for bool {
    fn to_record(&self) -> DataRecord {
        DataRecord::Flag(*self)
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match record {
            DataRecord::Flag(value) => Ok(value),
            other => Err(ProjectionError::WrongPayload(id, "Flag", other.kind_name())),
        }
    }
}
