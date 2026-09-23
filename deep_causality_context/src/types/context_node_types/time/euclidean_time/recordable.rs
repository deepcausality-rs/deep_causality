/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EuclideanTime, Temporal};
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, TimeRecord};
use deep_causality_num::FromPrimitive;

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<TimeRecord> for EuclideanTime<R> {
    fn to_record(&self) -> Result<TimeRecord, ProjectionError> {
        Ok(TimeRecord::Euclidean {
            scale: self.time_scale(),
            value: self.time_unit().into(),
        })
    }

    fn from_record(id: ContextoidId, record: TimeRecord) -> Result<Self, ProjectionError> {
        match record {
            TimeRecord::Euclidean { scale, value } => R::from_f64(value)
                .map(|unit| EuclideanTime::new(id, scale, unit))
                .ok_or(ProjectionError::Scalar(id, value)),
            other => Err(ProjectionError::WrongVariant(
                id,
                "Euclidean",
                other.kind_name(),
            )),
        }
    }
}
