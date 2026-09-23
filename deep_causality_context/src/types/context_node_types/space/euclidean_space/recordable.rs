/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EuclideanSpace;
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceRecord};
use deep_causality_num::FromPrimitive;

fn lift<R: FromPrimitive>(id: ContextoidId, value: f64) -> Result<R, ProjectionError> {
    R::from_f64(value).ok_or(ProjectionError::Scalar(id, value))
}

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceRecord> for EuclideanSpace<R> {
    fn to_record(&self) -> Result<SpaceRecord, ProjectionError> {
        Ok(SpaceRecord::Euclidean {
            x: self.x().into(),
            y: self.y().into(),
            z: self.z().into(),
        })
    }

    fn from_record(id: ContextoidId, record: SpaceRecord) -> Result<Self, ProjectionError> {
        match record {
            SpaceRecord::Euclidean { x, y, z } => Ok(EuclideanSpace::new(
                id,
                lift(id, x)?,
                lift(id, y)?,
                lift(id, z)?,
            )),
            other => Err(ProjectionError::WrongVariant(
                id,
                "Euclidean",
                other.kind_name(),
            )),
        }
    }
}
