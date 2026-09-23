/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

impl Storable for String {
    fn to_record(&self) -> DataRecord {
        DataRecord::Text(self.clone())
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match record {
            DataRecord::Text(text) => Ok(text),
            other => Err(ProjectionError::WrongPayload(id, "Text", other.kind_name())),
        }
    }
}
