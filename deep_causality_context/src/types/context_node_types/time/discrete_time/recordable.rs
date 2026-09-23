/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DiscreteTime, Temporal};
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, TimeRecord};

impl Recordable<TimeRecord> for DiscreteTime {
    fn to_record(&self) -> Result<TimeRecord, ProjectionError> {
        Ok(TimeRecord::Discrete {
            scale: self.time_scale(),
            tick: self.time_unit(),
        })
    }

    fn from_record(id: ContextoidId, record: TimeRecord) -> Result<Self, ProjectionError> {
        match record {
            TimeRecord::Discrete { scale, tick } => Ok(DiscreteTime::new(id, scale, tick)),
            other => Err(ProjectionError::WrongVariant(
                id,
                "Discrete",
                other.kind_name(),
            )),
        }
    }
}
