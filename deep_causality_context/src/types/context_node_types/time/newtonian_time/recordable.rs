/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::lift_scalar::lift_scalar;
use crate::{NewtonianTime, Temporal};
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, TimeRecord};
use deep_causality_num::FromPrimitive;

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<TimeRecord> for NewtonianTime<R> {
    fn to_record(&self) -> Result<TimeRecord, ProjectionError> {
        Ok(TimeRecord::Newtonian {
            scale: self.time_scale(),
            value: self.time_unit().into(),
        })
    }

    fn from_record(id: ContextoidId, record: TimeRecord) -> Result<Self, ProjectionError> {
        match record {
            TimeRecord::Newtonian { scale, value } => {
                lift_scalar(id, value).map(|unit| NewtonianTime::new(id, scale, unit))
            }
            other => Err(ProjectionError::WrongVariant(
                id,
                "Newtonian",
                other.kind_name(),
            )),
        }
    }
}
