/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The one per-type fact a numerical draw needs.

use deep_causality_num::{BFloat16, Float106};

/// How many 53-bit words a scalar's significand absorbs.
///
/// This is the **only** per-type fact the sampling layer consults; everything else is algebra.
/// `deep_causality_fft`'s `FftScalar` is the pattern being followed: one capability trait,
/// blanket-shaped, so a new scalar joins by satisfying the algebra rather than by a new file of
/// hand-written implementations.
///
/// # Why the constant cannot be dropped
///
/// A single 53-bit draw returned as a `Float106` is an `f64` wearing a wider type. It satisfies
/// every bounds check and passes every moment test, and it silently defeats the precision claim
/// the alias discipline exists to make. Measured: a single-word generic carried a non-zero low
/// limb in **0 of 200** draws, while consuming the declared number of words carried one in
/// **400 of 400**.
///
/// The default serves every scalar whose significand is 53 bits or fewer, so `f32`, `f64` and
/// `BFloat16` are empty implementations and only the double-double states a value.
pub trait RandWidth {
    /// The number of 53-bit generator words this scalar's significand can absorb.
    const WORDS: u32 = 1;
}

impl RandWidth for f32 {}
impl RandWidth for f64 {}
impl RandWidth for BFloat16 {}

impl RandWidth for Float106 {
    // A double-double carries ~106 bits, which two 53-bit words fill.
    const WORDS: u32 = 2;
}
