/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::decision::CheckReport;
use crate::types::qpu::shot_estimate::ShotEstimate;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// A real-valued specification a read-out is judged against.
///
/// A threshold on a real quantity is a classical proposition, so the verdict it produces is
/// Boolean and takes no commutation test. The judgement is a margin over shots: the shortfall
/// against the spec, measured against the shot-noise width of the estimate, so both sources of
/// uncertainty are in it, sampling from the budget and rounding from the scalar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spec<R> {
    /// The estimate must reach `R` from below.
    AtLeast(R),
    /// The estimate must stay at or below `R`.
    AtMost(R),
}

impl<R> Spec<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// `estimate ≥ value`, within shot noise.
    pub fn at_least(value: R) -> Self {
        Self::AtLeast(value)
    }

    /// `estimate ≤ value`, within shot noise.
    pub fn at_most(value: R) -> Self {
        Self::AtMost(value)
    }

    /// The judgement, as the decision form.
    pub fn judge(&self, estimate: &ShotEstimate<R>) -> CheckReport<R> {
        match self {
            Self::AtLeast(v) => estimate.at_least(*v),
            Self::AtMost(v) => estimate.at_most(*v),
        }
    }
}
