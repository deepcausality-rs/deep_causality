/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;

mod iterator;

/// Identifiers a store has made unique and handed out for nodes not yet created.
///
/// Opaque except for `next`, its [`Iterator`] implementation: a caller sees a unique number and
/// never how it was made unique. A reserve is a lease with no return; an identifier handed out and
/// never used is never used. It is not `Clone`: a copy would hand the same identifiers out twice.
#[derive(Debug, PartialEq, Eq)]
pub struct IdReserve {
    ids: Vec<ContextoidId>,
    taken: usize,
}

impl IdReserve {
    pub const fn new(ids: Vec<ContextoidId>) -> Self {
        Self { ids, taken: 0 }
    }

    /// How many identifiers `next` can still hand out.
    pub const fn remaining(&self) -> usize {
        self.ids.len() - self.taken
    }

    pub(super) const fn taken(&self) -> usize {
        self.taken
    }

    pub(super) fn advance(&mut self) {
        self.taken += 1;
    }

    pub(super) fn ids(&self) -> &[ContextoidId] {
        &self.ids
    }
}
