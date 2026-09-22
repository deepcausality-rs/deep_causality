/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt::{Display, Formatter};

/// Where a data node's value is stored, for a context that outlives the process.
///
/// A data node's payload is ordinarily the reading itself. A context persisted to a graph store
/// carries the reference instead: the store holds what a thing is and what it relates to, and the
/// reading stays in the substrate that produced it. One number in two places is two numbers that
/// disagree the moment either is corrected.
///
/// The two fields are the two halves of a lookup — which store, and which row within it. They stay
/// separate because a reference joined into one string can no longer be grouped or filtered by
/// store.
#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
pub struct SubstrateRef {
    source: String,
    key: String,
}

impl SubstrateRef {
    /// Names the store and the row holding a value.
    #[must_use]
    pub const fn new(source: String, key: String) -> Self {
        Self { source, key }
    }

    /// The store holding the value.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// The row within that store.
    #[must_use]
    pub fn key(&self) -> &str {
        self.key.as_str()
    }
}

impl Display for SubstrateRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.source, self.key)
    }
}
