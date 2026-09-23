/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::GeoSpace;
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceRecord};
use deep_causality_num::FromPrimitive;

fn lift<R: FromPrimitive>(id: ContextoidId, value: f64) -> Result<R, ProjectionError> {
    R::from_f64(value).ok_or(ProjectionError::Scalar(id, value))
}

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
            } => Ok(GeoSpace::new(
                id,
                lift(id, lat)?,
                lift(id, lon)?,
                lift(id, alt)?,
                datum,
            )),
            other => Err(ProjectionError::WrongVariant(id, "Geo", other.kind_name())),
        }
    }
}
