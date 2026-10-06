/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EcefSpace;
use crate::utils::lift_scalar::lift_scalar;
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceRecord};
use deep_causality_num::FromPrimitive;

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceRecord> for EcefSpace<R> {
    fn to_record(&self) -> Result<SpaceRecord, ProjectionError> {
        Ok(SpaceRecord::Ecef {
            x: self.x().into(),
            y: self.y().into(),
            z: self.z().into(),
        })
    }

    fn from_record(id: ContextoidId, record: SpaceRecord) -> Result<Self, ProjectionError> {
        match record {
            SpaceRecord::Ecef { x, y, z } => Ok(EcefSpace::new(
                id,
                lift_scalar(id, x)?,
                lift_scalar(id, y)?,
                lift_scalar(id, z)?,
            )),
            other => Err(ProjectionError::WrongVariant(id, "Ecef", other.kind_name())),
        }
    }
}
