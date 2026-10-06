/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::lift_scalar::lift_scalar;
use crate::{GalileanSpacetime, SpaceTemporal, Temporal};
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceTimeRecord};
use deep_causality_num::FromPrimitive;

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceTimeRecord>
    for GalileanSpacetime<R>
{
    fn to_record(&self) -> Result<SpaceTimeRecord, ProjectionError> {
        Ok(SpaceTimeRecord::Galilean {
            t: (*self.t()).into(),
            x: self.x().into(),
            y: self.y().into(),
            z: self.z().into(),
            scale: self.time_scale(),
        })
    }

    fn from_record(id: ContextoidId, record: SpaceTimeRecord) -> Result<Self, ProjectionError> {
        match record {
            SpaceTimeRecord::Galilean { t, x, y, z, scale } => Ok(GalileanSpacetime::new(
                id,
                lift_scalar(id, x)?,
                lift_scalar(id, y)?,
                lift_scalar(id, z)?,
                lift_scalar(id, t)?,
                scale,
            )),
            other => Err(ProjectionError::WrongVariant(
                id,
                "Galilean",
                other.kind_name(),
            )),
        }
    }
}
