/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

/// A sequence is a `List` of its elements' records, in order.
impl<T: Storable> Storable for Vec<T> {
    fn to_record(&self) -> DataRecord {
        DataRecord::List(self.iter().map(Storable::to_record).collect())
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match record {
            DataRecord::List(items) => items
                .into_iter()
                .map(|item| T::from_record(id, item))
                .collect(),
            other => Err(ProjectionError::WrongPayload(id, "List", other.kind_name())),
        }
    }
}

/// An optional value is a `List` of zero or one element.
impl<T: Storable> Storable for Option<T> {
    fn to_record(&self) -> DataRecord {
        DataRecord::List(self.iter().map(Storable::to_record).collect())
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match record {
            DataRecord::List(mut items) => match items.len() {
                0 => Ok(None),
                1 => T::from_record(id, items.swap_remove(0)).map(Some),
                _ => Err(ProjectionError::WrongPayload(id, "Option", "List")),
            },
            other => Err(ProjectionError::WrongPayload(id, "List", other.kind_name())),
        }
    }
}
