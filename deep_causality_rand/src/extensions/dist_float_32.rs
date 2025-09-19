/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Distribution, Rng};
use crate::{Open01, OpenClosed01, StandardUniform};
use deep_causality_num::{FloatAsScalar, FloatFromInt, IntAsScalar, IntoFloat};
use std::mem;

impl Distribution<f32> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        // Multiply-based method; 24/53 random bits; [0, 1) interval.
        // We use the most significant bits because for simple RNGs
        // those are usually more random.
        let float_size = mem::size_of::<f32>() as u32 * 8;
        let precision = 23 + 1;
        let scale = 1.0 / ((1_u32 << precision) as f32);

        let value: u32 = rng.random();
        let value = value >> u32::splat(float_size - precision);
        f32::splat(scale) * f32::cast_from_int(value)
    }
}

impl Distribution<f32> for OpenClosed01 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        // Multiply-based method; 24/53 random bits; (0, 1] interval.
        // We use the most significant bits because for simple RNGs
        // those are usually more random.
        let float_size = mem::size_of::<f32>() as u32 * 8;
        let precision = 23 + 1;
        let scale = 1.0 / ((1_u32 << precision) as f32);

        let value: u32 = rng.random();
        let value = value >> u32::splat(float_size - precision);
        // Add 1 to shift up; will not overflow because of right-shift:
        f32::splat(scale) * f32::cast_from_int(value + u32::splat(1))
    }
}

impl Distribution<f32> for Open01 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        // Transmute-based method; 23/52 random bits; (0, 1) interval.
        // We use the most significant bits because for simple RNGs
        // those are usually more random.
        let float_size = mem::size_of::<f32>() as u32 * 8;

        let value: u32 = rng.random();
        let fraction = value >> u32::splat(float_size - 23);
        fraction.into_float_with_exponent(0) - f32::splat(1.0 - f32::EPSILON / 2.0)
    }
}
