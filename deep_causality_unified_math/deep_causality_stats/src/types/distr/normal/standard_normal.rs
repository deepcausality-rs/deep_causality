/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The standard normal, written once and generic in the scalar.

use crate::{Open01, RandWidth, StandardUniform};
use deep_causality_algebra::{Real, RealField};
use deep_causality_num::FromPrimitive;
use deep_causality_rand::{Distribution, Rng};

/// The standard normal distribution `N(0, 1)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardNormal;

/// Box–Muller in the caller's scalar.
///
/// This replaces an `f64` ziggurat that `f32` narrowed from and `Float106` could not use at all —
/// which is why the double-double needed its own hand-written Box–Muller beside it. One body now
/// serves every scalar, and the transcendentals come from `Real`, so a wide scalar is computed at
/// its own precision rather than widened from a narrow intermediate.
///
/// The ziggurat is faster at `f64`, avoiding two transcendentals per draw. Trading it away is a
/// real cost and not a free simplification; it is the right trade here because a ziggurat cannot
/// serve a scalar wider than the table it is built from, and precision as a parameter is the point
/// of this crate. A specialised `f64` path can return behind this same generic surface later, as a
/// measured optimisation rather than a starting assumption.
///
/// It also removes an unbounded loop. The ziggurat's tail is a rejection sampler with no iteration
/// cap, so on a degenerate generator it does not terminate; Box–Muller has no rejection on its
/// main path.
impl<T> Distribution<T> for StandardNormal
where
    T: RealField + FromPrimitive + RandWidth,
    Open01: Distribution<T>,
    StandardUniform: Distribution<T>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        let two = T::from_u64(2).expect("2 is representable in every supported scalar");
        // `u1` from the open interval: `ln(0)` is an infinity no check on the result would catch.
        let u1: T = Open01.sample(rng);
        let u2: T = StandardUniform.sample(rng);
        let radius = Real::sqrt(-two * Real::ln(u1));
        let theta = two * T::pi() * u2;
        radius * Real::cos(theta)
    }
}
