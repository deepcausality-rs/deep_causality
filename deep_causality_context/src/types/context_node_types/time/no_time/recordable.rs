/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NoTime;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, TimeRecord};
use deep_causality_core::Identifiable;

/// The absent time has no record: writing it is `Unrecordable`, reading any record into it is
/// `WrongVariant`.
impl Recordable<TimeRecord> for NoTime {
    fn to_record(&self) -> Result<TimeRecord, ProjectionError> {
        Err(ProjectionError::Unrecordable(self.id(), "NoTime"))
    }

    fn from_record(id: ContextoidId, record: TimeRecord) -> Result<Self, ProjectionError> {
        Err(ProjectionError::WrongVariant(
            id,
            "NoTime",
            record.kind_name(),
        ))
    }
}
