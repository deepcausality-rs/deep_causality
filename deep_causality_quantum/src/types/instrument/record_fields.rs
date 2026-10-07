/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reading a payload struct back from its `Fields` record.

use crate::QuantumError;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use deep_causality_context::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

/// The entries of `record`, which must be a `Fields` record.
pub(crate) fn entries(
    id: ContextoidId,
    record: DataRecord,
) -> Result<Vec<(String, DataRecord)>, ProjectionError> {
    match record {
        DataRecord::Fields(entries) => Ok(entries),
        other => Err(ProjectionError::WrongPayload(
            id,
            "Fields",
            other.kind_name(),
        )),
    }
}

/// The value under `name` among `entries`, read as `T`.
pub(crate) fn field<T: Storable>(
    id: ContextoidId,
    entries: &[(String, DataRecord)],
    name: &'static str,
) -> Result<T, ProjectionError> {
    let record = entries
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
        .ok_or(ProjectionError::MissingField(id, name))?;
    T::from_record(id, record)
}

/// A payload rule the record's values break, as the projection error for the node under `id`.
pub(crate) fn rejected(id: ContextoidId, error: QuantumError) -> ProjectionError {
    ProjectionError::Rejected(id, error.to_string())
}
