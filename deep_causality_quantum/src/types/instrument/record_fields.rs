/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reading a payload struct back from its `Fields` record.

use crate::QuantumError;
use alloc::format;
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

/// The value under `name` among `entries`, read as `T`. A name carried by two entries is
/// rejected before either value is read, since the record does not say which one holds.
pub(crate) fn field<T: Storable>(
    id: ContextoidId,
    entries: &[(String, DataRecord)],
    name: &'static str,
) -> Result<T, ProjectionError> {
    let mut named = entries.iter().filter(|(key, _)| key == name);
    let (_, record) = named
        .next()
        .ok_or(ProjectionError::MissingField(id, name))?;
    if named.next().is_some() {
        return Err(ProjectionError::Rejected(
            id,
            format!("a Fields record names '{name}' more than once"),
        ));
    }
    T::from_record(id, record.clone())
}

/// A payload rule the record's values break, as the projection error for the node under `id`.
pub(crate) fn rejected(id: ContextoidId, error: QuantumError) -> ProjectionError {
    ProjectionError::Rejected(id, error.to_string())
}
