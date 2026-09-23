/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Data, Storable};
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError, Recordable};

/// Every storable payload makes its data node recordable. This is the one place a payload's
/// `Storable` meets the node's `Recordable`.
impl<T: Storable + Default + Clone + PartialEq> Recordable<DataRecord> for Data<T> {
    fn to_record(&self) -> Result<DataRecord, ProjectionError> {
        Ok(self.data.to_record())
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        T::from_record(id, record).map(|data| Data::new(id, data))
    }
}
