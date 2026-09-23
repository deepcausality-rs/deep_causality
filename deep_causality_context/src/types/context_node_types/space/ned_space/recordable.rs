/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NedSpace;
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceRecord};
use deep_causality_num::FromPrimitive;

fn lift<R: FromPrimitive>(id: ContextoidId, value: f64) -> Result<R, ProjectionError> {
    R::from_f64(value).ok_or(ProjectionError::Scalar(id, value))
}

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
                lift(id, north)?,
                lift(id, east)?,
                lift(id, down)?,
            )),
            other => Err(ProjectionError::WrongVariant(id, "Ned", other.kind_name())),
        }
    }
}
