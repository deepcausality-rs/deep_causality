/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{LorentzianSpacetime, SpaceTemporal, Temporal};
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceTimeRecord};
use deep_causality_num::FromPrimitive;

fn lift<R: FromPrimitive>(id: ContextoidId, value: f64) -> Result<R, ProjectionError> {
    R::from_f64(value).ok_or(ProjectionError::Scalar(id, value))
}

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceTimeRecord>
    for LorentzianSpacetime<R>
{
    fn to_record(&self) -> Result<SpaceTimeRecord, ProjectionError> {
        Ok(SpaceTimeRecord::Lorentzian {
            x: self.x().into(),
            y: self.y().into(),
            z: self.z().into(),
            t: (*self.t()).into(),
            scale: self.time_scale(),
        })
    }

    fn from_record(id: ContextoidId, record: SpaceTimeRecord) -> Result<Self, ProjectionError> {
        match record {
            SpaceTimeRecord::Lorentzian { x, y, z, t, scale } => Ok(LorentzianSpacetime::new(
                id,
                lift(id, x)?,
                lift(id, y)?,
                lift(id, z)?,
                lift(id, t)?,
                scale,
            )),
            other => Err(ProjectionError::WrongVariant(
                id,
                "Lorentzian",
                other.kind_name(),
            )),
        }
    }
}
