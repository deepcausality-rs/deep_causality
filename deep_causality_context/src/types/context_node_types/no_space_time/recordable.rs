/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NoSpaceTime;
use deep_causality_algebra::RealField;
use deep_causality_context_store::{
    ContextoidId, ProjectionError, Recordable, SpaceRecord, SpaceTimeRecord,
};
use deep_causality_core::Identifiable;

/// The absent spatial extent has no record: writing it is `Unrecordable`, reading any record
/// into it is `WrongVariant`. Two implementations, one per `Context` slot it fills.
impl<R: RealField> Recordable<SpaceRecord> for NoSpaceTime<R> {
    fn to_record(&self) -> Result<SpaceRecord, ProjectionError> {
        Err(ProjectionError::Unrecordable(self.id(), "NoSpaceTime"))
    }

    fn from_record(id: ContextoidId, record: SpaceRecord) -> Result<Self, ProjectionError> {
        Err(ProjectionError::WrongVariant(
            id,
            "NoSpaceTime",
            record.kind_name(),
        ))
    }
}

impl<R: RealField> Recordable<SpaceTimeRecord> for NoSpaceTime<R> {
    fn to_record(&self) -> Result<SpaceTimeRecord, ProjectionError> {
        Err(ProjectionError::Unrecordable(self.id(), "NoSpaceTime"))
    }

    fn from_record(id: ContextoidId, record: SpaceTimeRecord) -> Result<Self, ProjectionError> {
        Err(ProjectionError::WrongVariant(
            id,
            "NoSpaceTime",
            record.kind_name(),
        ))
    }
}
