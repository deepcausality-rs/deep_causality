/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Family cache key.
//!
//! Posterior assembly scores each unique family `(node, parents)` once. The key
//! is the node plus its **sorted, deduplicated** parent set, so it is independent
//! of the order parents happen to appear in a sampled DAG.

/// The canonical key for a family: the node and its sorted parent indices.
pub type FamilyKey = (usize, Vec<usize>);

/// Returns the canonical [`FamilyKey`] for `(node, parents)`, sorting and
/// deduplicating the parents so order does not matter.
pub fn family_key(node: usize, parents: &[usize]) -> FamilyKey {
    let mut p: Vec<usize> = parents.to_vec();
    p.sort_unstable();
    p.dedup();
    (node, p)
}
