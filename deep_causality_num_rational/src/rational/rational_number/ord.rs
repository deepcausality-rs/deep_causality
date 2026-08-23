/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Equality and ordering.
//!
//! ℚ is a totally ordered field. Because every `Rational` is held in canonical form, equality is
//! structural — no cross-multiplication is needed, and `PartialEq` cannot disagree with `Ord`.

use super::{Rational, RationalScalar};
use core::cmp::Ordering;

impl<T: RationalScalar> PartialEq for Rational<T> {
    /// Canonical form makes this exact: `2/4` and `1/2` are both stored as `1/2`.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.numer() == other.numer() && self.denom() == other.denom()
    }
}

impl<T: RationalScalar> Eq for Rational<T> {}

impl<T: RationalScalar> PartialOrd for Rational<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: RationalScalar> Ord for Rational<T> {
    /// Compares `a/b` against `c/d` as `a·d` against `c·b`.
    ///
    /// Both denominators are strictly positive, so cross-multiplying preserves the direction of
    /// the inequality — which is exactly why invariant 1 is worth maintaining.
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = *self.numer() * *other.denom();
        let rhs = *other.numer() * *self.denom();
        // `T: PartialOrd`, and the integers are totally ordered, so the comparison is decisive.
        if lhs < rhs {
            Ordering::Less
        } else if lhs > rhs {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
