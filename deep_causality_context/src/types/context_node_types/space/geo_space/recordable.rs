/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::GeoSpace;
use crate::utils::lift_scalar::lift_scalar;
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceRecord};
use deep_causality_num::FromPrimitive;

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceRecord> for GeoSpace<R> {
    fn to_record(&self) -> Result<SpaceRecord, ProjectionError> {
        Ok(SpaceRecord::Geo {
            lat: self.lat().into(),
            lon: self.lon().into(),
            alt: self.alt().into(),
            datum: self.datum(),
        })
    }

    fn from_record(id: ContextoidId, record: SpaceRecord) -> Result<Self, ProjectionError> {
        match record {
            SpaceRecord::Geo {
                lat,
                lon,
                alt,
                datum,
            } => GeoSpace::new(
                id,
                lift_scalar(id, lat)?,
                lift_scalar(id, lon)?,
                lift_scalar(id, alt)?,
                datum,
            )
            .map_err(|e| ProjectionError::Rejected(id, e.0)),
            other => Err(ProjectionError::WrongVariant(id, "Geo", other.kind_name())),
        }
    }
}
