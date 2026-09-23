/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::DataRecord;
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

mod substrate;

/// An in-memory substrate: values keyed by the node that carries them, under the source `"memory"`.
///
/// A second deposit under the same node replaces the first, because a node carries one value.
pub struct MemorySubstrate {
    values: Mutex<BTreeMap<String, DataRecord>>,
}

impl MemorySubstrate {
    /// The source every reference this substrate hands out names.
    pub const SOURCE: &'static str = "memory";

    pub fn new() -> Self {
        Self {
            values: Mutex::new(BTreeMap::new()),
        }
    }

    /// How many values the substrate holds.
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, BTreeMap<String, DataRecord>> {
        self.values.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for MemorySubstrate {
    fn default() -> Self {
        Self::new()
    }
}
