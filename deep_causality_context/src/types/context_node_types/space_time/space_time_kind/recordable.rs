/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EuclideanSpacetime, LorentzianSpacetime, SpaceTimeKind, TangentSpacetime};
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceTimeRecord};
use deep_causality_num::FromPrimitive;

/// Total: three variants, three arms, each delegating to the type it wraps.
impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceTimeRecord> for SpaceTimeKind<R> {
    fn to_record(&self) -> Result<SpaceTimeRecord, ProjectionError> {
        match self {
            SpaceTimeKind::Euclidean(spacetime) => spacetime.to_record(),
            SpaceTimeKind::Lorentzian(spacetime) => spacetime.to_record(),
            SpaceTimeKind::Tangent(spacetime) => spacetime.to_record(),
        }
    }

    fn from_record(id: ContextoidId, record: SpaceTimeRecord) -> Result<Self, ProjectionError> {
        match record {
            SpaceTimeRecord::Euclidean { .. } => {
                EuclideanSpacetime::from_record(id, record).map(SpaceTimeKind::Euclidean)
            }
            SpaceTimeRecord::Lorentzian { .. } => {
                LorentzianSpacetime::from_record(id, record).map(SpaceTimeKind::Lorentzian)
            }
            SpaceTimeRecord::Tangent { .. } => {
                TangentSpacetime::from_record(id, record).map(SpaceTimeKind::Tangent)
            }
        }
    }
}
