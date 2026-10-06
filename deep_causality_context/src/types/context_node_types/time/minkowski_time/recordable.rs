/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::lift_scalar::lift_scalar;
use crate::{MinkowskiTime, Temporal};
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, TimeRecord};
use deep_causality_num::FromPrimitive;

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<TimeRecord> for MinkowskiTime<R> {
    fn to_record(&self) -> Result<TimeRecord, ProjectionError> {
        Ok(TimeRecord::Minkowski {
            scale: self.time_scale(),
            value: self.time_unit().into(),
        })
    }

    fn from_record(id: ContextoidId, record: TimeRecord) -> Result<Self, ProjectionError> {
        match record {
            TimeRecord::Minkowski { scale, value } => {
                lift_scalar(id, value).map(|unit| MinkowskiTime::new(id, scale, unit))
            }
            other => Err(ProjectionError::WrongVariant(
                id,
                "Minkowski",
                other.kind_name(),
            )),
        }
    }
}
