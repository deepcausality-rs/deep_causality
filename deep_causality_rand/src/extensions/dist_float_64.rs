/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Distribution, Rng};
use crate::{Open01, OpenClosed01, StandardUniform};
use deep_causality_num::{FloatAsScalar, FloatFromInt, IntAsScalar, IntoFloat};
use std::mem;

impl Distribution<f64> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let float_size = mem::size_of::<f64>() as u64 * 8;
        let precision = 52 + 1;
        let scale = 1.0 / ((1_u64 << precision) as f64);

        let value: u64 = rng.random();
        let value = value >> u64::splat(float_size - precision);
        f64::splat(scale) * f64::cast_from_int(value)
    }
}

impl Distribution<f64> for OpenClosed01 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let float_size = mem::size_of::<f64>() as u64 * 8;
        let precision = 52 + 1;
        let scale = 1.0 / ((1_u64 << precision) as f64);

        let value: u64 = rng.random();
        let value = value >> u64::splat(float_size - precision);
        f64::splat(scale) * f64::cast_from_int(value + u64::splat(1))
    }
}

impl Distribution<f64> for Open01 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let float_size = mem::size_of::<f64>() as u64 * 8;

        let value: u64 = rng.random();
        let fraction = value >> u64::splat(float_size - 52);
        fraction.into_float_with_exponent(0) - f64::splat(1.0 - f64::EPSILON / 2.0)
    }
}
