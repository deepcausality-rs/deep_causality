/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Family log-likelihood cache.
//!
//! During posterior assembly (`brcd_update`) the same family `(node, parents)`
//! recurs across many sampled DAGs. The authoritative code computes each unique
//! family's per-row log-likelihood **once** and reuses it; this cache replicates
//! that — the dominant performance lever — keyed on the node and its **sorted**
//! parent set so the key is independent of parent order.

use crate::brcd::brcd_error::BrcdError;
use std::collections::BTreeMap;

/// The canonical cache key for a family: the node and its sorted parent indices.
pub type FamilyKey = (usize, Vec<usize>);

/// Returns the canonical [`FamilyKey`] for `(node, parents)`, sorting and
/// deduplicating the parents so order does not matter.
pub fn family_key(node: usize, parents: &[usize]) -> FamilyKey {
    let mut p: Vec<usize> = parents.to_vec();
    p.sort_unstable();
    p.dedup();
    (node, p)
}

/// A cache of per-row family log-likelihoods keyed by [`FamilyKey`].
#[derive(Debug, Clone, Default)]
pub struct FamilyCache<T> {
    entries: BTreeMap<FamilyKey, Vec<T>>,
}

impl<T: Clone> FamilyCache<T> {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Number of cached families.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the cache holds no families.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the cached per-row log-likelihoods for `(node, parents)`, or
    /// `None` if the family has not been computed yet.
    pub fn get(&self, node: usize, parents: &[usize]) -> Option<&[T]> {
        self.entries
            .get(&family_key(node, parents))
            .map(Vec::as_slice)
    }

    /// Returns the cached per-row log-likelihoods for `(node, parents)`,
    /// computing and storing them with `compute` on the first request. The
    /// `compute` closure runs at most once per distinct family.
    ///
    /// # Errors
    /// Propagates any [`BrcdError`] returned by `compute` (nothing is cached on
    /// failure).
    pub fn get_or_try_insert_with<F>(
        &mut self,
        node: usize,
        parents: &[usize],
        compute: F,
    ) -> Result<&[T], BrcdError>
    where
        F: FnOnce() -> Result<Vec<T>, BrcdError>,
    {
        let key = family_key(node, parents);
        if !self.entries.contains_key(&key) {
            let value = compute()?;
            self.entries.insert(key.clone(), value);
        }
        Ok(self.entries.get(&key).expect("just inserted").as_slice())
    }
}
