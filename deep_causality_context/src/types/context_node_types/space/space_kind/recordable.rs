/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EcefSpace, EuclideanSpace, GeoSpace, NedSpace, SpaceKind};
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceRecord};
use deep_causality_num::FromPrimitive;

/// Total: four variants, four arms, each delegating to the type it wraps.
impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceRecord> for SpaceKind<R> {
    fn to_record(&self) -> Result<SpaceRecord, ProjectionError> {
        match self {
            SpaceKind::Geo(space) => space.to_record(),
            SpaceKind::Ecef(space) => space.to_record(),
            SpaceKind::Euclidean(space) => space.to_record(),
            SpaceKind::Ned(space) => space.to_record(),
        }
    }

    fn from_record(id: ContextoidId, record: SpaceRecord) -> Result<Self, ProjectionError> {
        match record {
            SpaceRecord::Geo { .. } => GeoSpace::from_record(id, record).map(SpaceKind::Geo),
            SpaceRecord::Ecef { .. } => EcefSpace::from_record(id, record).map(SpaceKind::Ecef),
            SpaceRecord::Euclidean { .. } => {
                EuclideanSpace::from_record(id, record).map(SpaceKind::Euclidean)
            }
            SpaceRecord::Ned { .. } => NedSpace::from_record(id, record).map(SpaceKind::Ned),
        }
    }
}
