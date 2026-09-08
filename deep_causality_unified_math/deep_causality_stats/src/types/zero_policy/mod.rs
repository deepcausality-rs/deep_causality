/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! How an information measure treats entries at or near zero.

/// Which entries are omitted from the sum.
///
/// `lim(p → 0) p·log p = 0`, so a zero entry contributes nothing and the only question is where
/// the cutoff sits. That is a parameter because the shipped implementations put it in two places:
/// one skips at exactly zero, the other at a small threshold after normalising.
///
/// A *negative* entry is not covered here. It is not a probability, no cutoff interprets it, and
/// every function in this crate rejects it with a typed error.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ZeroPolicy<T> {
    /// Omit entries that are exactly zero. Anything strictly positive contributes.
    #[default]
    SkipZero,
    /// Omit entries at or below `threshold`.
    ///
    /// Used where the distribution has been normalised and entries below the threshold are
    /// numerical residue rather than mass.
    SkipBelow(T),
}
