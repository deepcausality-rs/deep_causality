/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EntropicTime, Temporal};
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, TimeRecord};

impl Recordable<TimeRecord> for EntropicTime {
    fn to_record(&self) -> Result<TimeRecord, ProjectionError> {
        Ok(TimeRecord::Entropic {
            tick: self.time_unit(),
        })
    }

    fn from_record(id: ContextoidId, record: TimeRecord) -> Result<Self, ProjectionError> {
        match record {
            TimeRecord::Entropic { tick } => Ok(EntropicTime::new(id, tick)),
            other => Err(ProjectionError::WrongVariant(
                id,
                "Entropic",
                other.kind_name(),
            )),
        }
    }
}
