/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    GalileanSpacetime, MinkowskiSpacetime, NewtonianSpacetime, SpaceTimeKind, TangentSpacetime,
};
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceTimeRecord};
use deep_causality_num::FromPrimitive;

/// Total: four variants, four arms, each delegating to the type it wraps.
impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceTimeRecord> for SpaceTimeKind<R> {
    fn to_record(&self) -> Result<SpaceTimeRecord, ProjectionError> {
        match self {
            SpaceTimeKind::Galilean(spacetime) => spacetime.to_record(),
            SpaceTimeKind::Newtonian(spacetime) => spacetime.to_record(),
            SpaceTimeKind::Minkowski(spacetime) => spacetime.to_record(),
            SpaceTimeKind::Tangent(spacetime) => spacetime.to_record(),
        }
    }

    fn from_record(id: ContextoidId, record: SpaceTimeRecord) -> Result<Self, ProjectionError> {
        match record {
            SpaceTimeRecord::Galilean { .. } => {
                GalileanSpacetime::from_record(id, record).map(SpaceTimeKind::Galilean)
            }
            SpaceTimeRecord::Newtonian { .. } => {
                NewtonianSpacetime::from_record(id, record).map(SpaceTimeKind::Newtonian)
            }
            SpaceTimeRecord::Minkowski { .. } => {
                MinkowskiSpacetime::from_record(id, record).map(SpaceTimeKind::Minkowski)
            }
            SpaceTimeRecord::Tangent { .. } => {
                TangentSpacetime::from_record(id, record).map(SpaceTimeKind::Tangent)
            }
        }
    }
}
