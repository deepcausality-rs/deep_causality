/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NedSpace;
use crate::utils::lift_scalar::lift_scalar;
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceRecord};
use deep_causality_num::FromPrimitive;

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceRecord> for NedSpace<R> {
    fn to_record(&self) -> Result<SpaceRecord, ProjectionError> {
        Ok(SpaceRecord::Ned {
            north: self.north().into(),
            east: self.east().into(),
            down: self.down().into(),
        })
    }

    fn from_record(id: ContextoidId, record: SpaceRecord) -> Result<Self, ProjectionError> {
        match record {
            SpaceRecord::Ned { north, east, down } => Ok(NedSpace::new(
                id,
                lift_scalar(id, north)?,
                lift_scalar(id, east)?,
                lift_scalar(id, down)?,
            )),
            other => Err(ProjectionError::WrongVariant(id, "Ned", other.kind_name())),
        }
    }
}
