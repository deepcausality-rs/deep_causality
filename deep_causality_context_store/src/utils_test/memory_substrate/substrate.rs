/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils_test::MemorySubstrate;
use crate::{ContextoidId, DataRecord, MemorySubstrateError, Substrate, SubstrateRef};
use core::future::ready;

impl Substrate for MemorySubstrate {
    type Error = MemorySubstrateError;

    fn deposit(
        &self,
        node: ContextoidId,
        value: &DataRecord,
    ) -> impl Future<Output = Result<SubstrateRef, Self::Error>> + Send {
        if matches!(value, DataRecord::Reference(_)) {
            return ready(Err(MemorySubstrateError::ReferenceRefused(node)));
        }
        let key = node.to_string();
        self.lock().insert(key.clone(), value.clone());
        ready(Ok(SubstrateRef::new(Self::SOURCE.to_string(), key)))
    }

    fn resolve(
        &self,
        reference: &SubstrateRef,
    ) -> impl Future<Output = Result<DataRecord, Self::Error>> + Send {
        let held = if reference.source() == Self::SOURCE {
            self.lock().get(reference.key()).cloned()
        } else {
            None
        };
        ready(held.ok_or_else(|| MemorySubstrateError::UnknownReference(reference.clone())))
    }
}
