/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

/// A data payload that one `DataRecord` describes completely, in both directions.
///
/// A payload type implements this once, and `Data<T>` is then persistable wherever a `Data<f64>`
/// is: `snapshot`, `restore`, the store and the stream all go through the blanket
/// `Recordable<DataRecord>` for `Data<T: Storable>`. A struct maps to `DataRecord::Fields`, a
/// sequence to `List`, a scalar to its arm.
///
/// Declared here rather than in the store crate because the software scalars live in
/// `deep_causality_num`, which is foreign to both, and only the crate that declares a trait may
/// implement it for a foreign type. A backend never names this trait; it sees `DataRecord`.
pub trait Storable: Sized {
    /// The payload as a record.
    fn to_record(&self) -> DataRecord;

    /// The payload a record describes, for the node under `id`, or why it cannot be.
    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError>;
}
