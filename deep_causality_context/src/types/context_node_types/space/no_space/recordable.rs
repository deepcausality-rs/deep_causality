/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NoSpace;
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceRecord};
use deep_causality_core::Identifiable;

/// The absent position has no record: writing it is `Unrecordable`, reading any record into it
/// is `WrongVariant`.
impl<R: RealField> Recordable<SpaceRecord> for NoSpace<R> {
    fn to_record(&self) -> Result<SpaceRecord, ProjectionError> {
        Err(ProjectionError::Unrecordable(self.id(), "NoSpace"))
    }

    fn from_record(id: ContextoidId, record: SpaceRecord) -> Result<Self, ProjectionError> {
        Err(ProjectionError::WrongVariant(
            id,
            "NoSpace",
            record.kind_name(),
        ))
    }
}
