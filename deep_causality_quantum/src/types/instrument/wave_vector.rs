/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::string::ToString;
use deep_causality_context::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

/// The direction of the effective wave vector along the sensitive axis. Reversing it between
/// shots is the k-reversal configuration change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaveVector {
    /// Pointing up.
    Up,
    /// Pointing down.
    Down,
}

/// Recorded as the text `up` or `down`.
impl Storable for WaveVector {
    fn to_record(&self) -> DataRecord {
        let text = match self {
            Self::Up => "up",
            Self::Down => "down",
        };
        DataRecord::Text(text.to_string())
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match record {
            DataRecord::Text(text) if text == "up" => Ok(Self::Up),
            DataRecord::Text(text) if text == "down" => Ok(Self::Down),
            DataRecord::Text(text) => Err(ProjectionError::Rejected(
                id,
                alloc::format!("a wave vector is 'up' or 'down', got '{text}'"),
            )),
            other => Err(ProjectionError::WrongPayload(id, "Text", other.kind_name())),
        }
    }
}
