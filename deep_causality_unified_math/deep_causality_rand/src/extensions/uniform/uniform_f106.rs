/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::distr::uniform::{RandFloat, UniformFloat};
use crate::{Distribution, Rng, SampleUniform, StandardWord};
use deep_causality_num::Float106;

impl RandFloat for Float106 {
    fn rand_float_gen<R: Rng + ?Sized>(rng: &mut R) -> Float106 {
        // A 53-bit high part plus an independent 53-bit low part scaled by `2^-53`: ~106 bits of
        // entropy, not an `f64` draw widened to double-double. Built from two machine words
        // directly, so the range machinery owes nothing to the distribution layer.
        const SCALE: f64 = 1.0 / ((1_u64 << 53) as f64);
        let hi: u64 = StandardWord.sample(rng);
        let lo: u64 = StandardWord.sample(rng);
        let hi = (hi >> 11) as f64 * SCALE;
        let lo = (lo >> 11) as f64 * SCALE;
        Float106::from(hi) + Float106::from(lo * SCALE)
    }
}

impl SampleUniform for Float106 {
    type Sampler = UniformFloat<Float106>;
}
