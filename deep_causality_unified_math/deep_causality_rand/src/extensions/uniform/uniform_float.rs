/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The float scalars the range sampler serves.
//!
//! Three files became this one. The draw itself is a single generic body on
//! [`RandFloat`](crate::RandFloat); what remains here is the one fact each scalar declares — how
//! many generator words its significand absorbs — and the `SampleUniform` binding.
//!
//! The bindings cannot be generic. `SampleUniform` is this crate's trait and `f64` is a foreign
//! type, so only this crate may write the implementation; and a blanket over `RealField` would
//! collide with the integer bindings beside it, which is the same `E0119` that shaped the whole
//! retrofit. They are one line each and carry no logic.

use crate::types::distr::uniform::{RandFloat, UniformFloat};
use crate::SampleUniform;
use deep_causality_num::Float106;

impl RandFloat for f32 {}
impl RandFloat for f64 {}

impl RandFloat for Float106 {
    // A double-double carries ~106 bits, which two 53-bit words fill. One word would make it an
    // `f64` wearing a wider type.
    const WORDS: u32 = 2;
}

impl SampleUniform for f32 {
    type Sampler = UniformFloat<f32>;
}

impl SampleUniform for f64 {
    type Sampler = UniformFloat<f64>;
}

impl SampleUniform for Float106 {
    type Sampler = UniformFloat<Float106>;
}
