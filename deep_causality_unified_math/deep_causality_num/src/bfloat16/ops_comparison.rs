/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Comparison for `BFloat16`, with IEEE 754 semantics: `+0.0 == -0.0`, and a NaN is unordered
//! and unequal to everything, itself included. Both are the `f32` comparisons on the widened
//! values, which is why the implementation does not compare bit patterns.

use crate::BFloat16;
use core::cmp::Ordering;

impl PartialEq for BFloat16 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.to_f32() == other.to_f32()
    }
}

impl PartialOrd for BFloat16 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_f32().partial_cmp(&other.to_f32())
    }
}
