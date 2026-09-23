/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};
use deep_causality_num::{BFloat16, Float106};

/// Two `f64` halves under the names `hi` and `lo`, restored as they were written, so the round
/// trip is exact for every representation, a non-finite half included.
impl Storable for Float106 {
    fn to_record(&self) -> DataRecord {
        DataRecord::Fields(vec![
            ("hi".to_string(), DataRecord::Number(self.hi())),
            ("lo".to_string(), DataRecord::Number(self.lo())),
        ])
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        let DataRecord::Fields(entries) = record else {
            return Err(ProjectionError::WrongPayload(
                id,
                "Fields",
                record.kind_name(),
            ));
        };
        let half = |name: &'static str| {
            entries
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| f64::from_record(id, value.clone()))
                .unwrap_or(Err(ProjectionError::MissingField(id, name)))
        };
        Ok(Float106::from_raw(half("hi")?, half("lo")?))
    }
}

/// Widened exactly into a `Number`; restored with the rounding its own arithmetic performs.
impl Storable for BFloat16 {
    fn to_record(&self) -> DataRecord {
        DataRecord::Number(f64::from(*self))
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        f64::from_record(id, record).map(BFloat16::from)
    }
}
