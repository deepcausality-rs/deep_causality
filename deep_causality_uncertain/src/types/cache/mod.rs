/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::RwLock;

/// A value produced during a sample run. Can be a float or a boolean.
#[derive(Debug, Clone, Copy)]
pub enum SampledValue {
    Float(f64),
    Bool(bool),
}

/// Key for the global sample cache: (Uncertain ID, Sample Index)
type SampleCacheKey = (usize, u64);

/// Thread-safe, global cache for sampled values.
pub struct GlobalSampleCache {
    data: RwLock<HashMap<SampleCacheKey, SampledValue>>,
}

impl GlobalSampleCache {
    /// Creates a new, empty cache.
    fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }

    /// Gets a value from the cache.
    pub fn get(&self, key: &SampleCacheKey) -> Option<SampledValue> {
        let reader = self.data.read().ok()?;
        reader.get(key).copied()
    }

    /// Inserts a value into the cache.
    pub fn insert(&self, key: SampleCacheKey, value: SampledValue) {
        if let Ok(mut writer) = self.data.write() {
            writer.insert(key, value);
        }
    }

    /// Gets a value from the cache, or computes it if not present, then caches and returns it.
    pub fn get_or_compute<F>(
        &self,
        key: SampleCacheKey,
        compute_fn: F,
    ) -> Result<SampledValue, crate::UncertainError>
    where
        F: FnOnce() -> Result<SampledValue, crate::UncertainError>,
    {
        // Try to get from cache first (read lock)
        if let Some(value) = self.get(&key) {
            return Ok(value);
        }

        // If not in cache, compute the value (need write lock)
        let value = compute_fn()?;

        // Insert into cache (write lock)
        self.insert(key, value);

        Ok(value)
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) {
        if let Ok(mut writer) = self.data.write() {
            writer.clear();
        }
    }
}

/// Global static instance of the cache, initialized once.
static GLOBAL_SAMPLE_CACHE: OnceLock<GlobalSampleCache> = OnceLock::new();

/// Returns a reference to the global sample cache.
pub fn get_global_cache() -> &'static GlobalSampleCache {
    GLOBAL_SAMPLE_CACHE.get_or_init(GlobalSampleCache::new)
}
