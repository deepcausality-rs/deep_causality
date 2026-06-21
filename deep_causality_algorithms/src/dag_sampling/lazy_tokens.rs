/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A depth-stamped boolean marker set with cheap save/restore.
//!
//! Ported verbatim from the authoritative `cliquepicking_rs::lazy_tokens`. The
//! clique-tree traversal repeatedly marks cliques as visited/considered and then
//! unwinds those marks when backtracking. Rather than copy a full boolean vector
//! at every recursion level, each marker carries the recursion `depth` at which
//! it was set; [`prepare`](LazyTokens::prepare) opens a new level and
//! [`restore`](LazyTokens::restore) rolls back exactly the marks made at the
//! current level.

/// A set of `usize` markers stamped with the recursion depth at which each was
/// set, enabling O(changes) save/restore.
#[derive(Debug)]
pub(crate) struct LazyTokens {
    /// `tokens[i] == depth` means index `i` is marked at the current level.
    tokens: Vec<usize>,
    /// Current recursion depth (the "current" stamp value).
    depth: usize,
    /// Per-level undo log: `(index, previous_stamp)` pairs to roll back.
    changed: Vec<Vec<(usize, usize)>>,
}

impl LazyTokens {
    /// Creates a marker set over `n` indices, all initially unmarked.
    pub(crate) fn new(n: usize) -> LazyTokens {
        LazyTokens {
            tokens: vec![0; n],
            depth: 0,
            changed: Vec::new(),
        }
    }

    /// Marks index `i` at the current depth (no-op if already marked), recording
    /// the previous stamp for later restore.
    pub(crate) fn set(&mut self, i: usize) {
        if self.tokens[i] == self.depth {
            return;
        }
        self.changed.last_mut().unwrap().push((i, self.tokens[i]));
        self.tokens[i] = self.depth;
    }

    /// Returns `true` if index `i` is marked at the current depth.
    pub(crate) fn check(&self, i: usize) -> bool {
        self.tokens[i] == self.depth
    }

    /// Opens a new recursion level, beginning a fresh undo log.
    pub(crate) fn prepare(&mut self) {
        self.depth += 1;
        self.changed.push(Vec::new());
    }

    /// Rolls back every mark made at the current level and closes it.
    pub(crate) fn restore(&mut self) {
        for &(i, prev_value) in self.changed.last().unwrap().iter() {
            self.tokens[i] = prev_value;
        }
        self.depth -= 1;
        self.changed.pop();
    }
}
