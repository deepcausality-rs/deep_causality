/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Storable, SubstrateRef};
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

impl Storable for SubstrateRef {
    fn to_record(&self) -> DataRecord {
        DataRecord::Reference(self.clone())
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match record {
            DataRecord::Reference(reference) => Ok(reference),
            other => Err(ProjectionError::WrongPayload(
                id,
                "Reference",
                other.kind_name(),
            )),
        }
    }
}
