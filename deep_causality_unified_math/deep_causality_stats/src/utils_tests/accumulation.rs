/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reach into the balanced-tree accumulator, for the suite that tests it.
//!
//! `PairwiseSum` is `pub(crate)`: it is how every
//! reduction in this crate forms a sum, and not something a caller constructs. The `tests` tree
//! cannot name it, so the two observations a test needs are taken here instead — the total, and
//! which slots hold a partial sum. Nothing else about the type is exposed, and the type itself
//! stays crate-private.

use crate::types::pairwise_sum::PairwiseSum;
use deep_causality_algebra::Real;

/// The balanced-tree sum of `values`, pushed in order.
pub fn pairwise_total<T: Real>(values: &[T]) -> T {
    let mut sum = PairwiseSum::new();
    for &v in values {
        sum.push(v);
    }
    sum.total()
}

/// Which slots hold a partial sum after pushing `values`, as a bit pattern.
///
/// The type's documented invariant is that this equals the number of observations: a slot holds
/// the sum of exactly `2ⁱ` of them, so occupancy is a binary counter. That is what makes a
/// streaming caller and a slice caller associate their additions identically, and it is a fact
/// about the arrangement rather than about any one total, so it is observed directly.
pub fn pairwise_occupancy<T: Real>(values: &[T]) -> u128 {
    let mut sum = PairwiseSum::new();
    for &v in values {
        sum.push(v);
    }
    sum.occupancy()
}
