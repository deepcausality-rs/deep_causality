/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::qpu::shot_estimate::ShotEstimate;
use alloc::format;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// A fringe read near its operating point: the read-out probability as a linear function of a
/// physical quantity, `p(x) = operating_point + slope · x`. An atom interferometer held at
/// mid-fringe has the operating point ½ and the slope `C · k_eff · T² / 2` per m/s².
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fringe<R> {
    operating_point: R,
    slope: R,
}

impl<R: RealField + FromPrimitive + core::fmt::Debug> Fringe<R> {
    /// The fringe through `operating_point` with `slope`, probability per unit of the quantity.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] when either is not finite;
    /// [`QuantumError::CalculationError`] when the operating point lies outside `(0, 1)` or the
    /// slope is zero, where the fringe reads nothing.
    pub fn new(operating_point: R, slope: R) -> Result<Self, QuantumError> {
        if !operating_point.is_finite() || !slope.is_finite() {
            return Err(QuantumError::NonFiniteValue(format!(
                "a fringe needs a finite operating point and slope, got {operating_point:?} and \
                 {slope:?}"
            )));
        }
        if operating_point <= R::zero() || operating_point >= R::one() {
            return Err(QuantumError::CalculationError(format!(
                "a fringe's operating point lies in (0, 1), got {operating_point:?}"
            )));
        }
        if slope == R::zero() {
            return Err(QuantumError::CalculationError(
                "a fringe of zero slope reads nothing".into(),
            ));
        }
        Ok(Self::at(operating_point, slope))
    }

    /// `value ± standard_error` as effective draws: the probability `p = p(value)`, and the draws
    /// `n = p(1−p) / (slope · standard_error)²` whose Bernoulli standard error is the published
    /// one carried onto the fringe.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] when the value or the standard error is not finite;
    /// [`QuantumError::CalculationError`] when the standard error is not positive, the value
    /// reads outside `(0, 1)` on this fringe, or the draws are not a finite positive number of
    /// the scalar, as when `slope · standard_error` is too large or too small for its square to
    /// be represented.
    pub fn effective_draws(
        &self,
        value: R,
        standard_error: R,
    ) -> Result<EffectiveDraws<R>, QuantumError> {
        if !value.is_finite() || !standard_error.is_finite() {
            return Err(QuantumError::NonFiniteValue(format!(
                "a published value and its standard error must be finite, got {value:?} and \
                 {standard_error:?}"
            )));
        }
        if standard_error <= R::zero() {
            return Err(QuantumError::CalculationError(format!(
                "a published standard error must be positive, got {standard_error:?}"
            )));
        }
        let probability = self.operating_point + self.slope * value;
        if probability <= R::zero() || probability >= R::one() {
            return Err(QuantumError::CalculationError(format!(
                "the value {value:?} reads {probability:?} on this fringe, outside (0, 1)"
            )));
        }
        let spread = self.slope * standard_error;
        let draws = probability * (R::one() - probability) / (spread * spread);
        if !draws.is_finite() || draws <= R::zero() {
            return Err(QuantumError::CalculationError(format!(
                "a standard error of {standard_error:?} on a fringe of slope {:?} gives \
                 {draws:?} effective draws, not a finite positive count",
                self.slope
            )));
        }
        Ok(EffectiveDraws { probability, draws })
    }

    /// The value and standard error `draws` stand for: the inverse of
    /// [`effective_draws`](Self::effective_draws), exact up to the rounding of the two.
    pub fn published(&self, draws: &EffectiveDraws<R>) -> (R, R) {
        (
            (draws.probability - self.operating_point) / self.slope,
            draws.standard_error() / self.slope.abs(),
        )
    }
}

impl<R: RealField> Fringe<R> {
    /// The fringe without the checks, for a caller whose invariants already hold them.
    pub(crate) fn at(operating_point: R, slope: R) -> Self {
        Self {
            operating_point,
            slope,
        }
    }

    /// The read-out probability at the operating point.
    pub fn operating_point(&self) -> R {
        self.operating_point
    }

    /// The change in read-out probability per unit of the quantity.
    pub fn slope(&self) -> R {
        self.slope
    }
}

/// A read-out probability and the effective Bernoulli draws behind it, which need not be whole.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectiveDraws<R> {
    probability: R,
    draws: R,
}

impl<R: RealField + FromPrimitive> EffectiveDraws<R> {
    /// The read-out probability.
    pub fn probability(&self) -> R {
        self.probability
    }

    /// The effective draws.
    pub fn draws(&self) -> R {
        self.draws
    }

    /// `√(p(1−p)/n)`, the read-out's standard error.
    pub fn standard_error(&self) -> R {
        (self.probability * (R::one() - self.probability) / self.draws).sqrt()
    }

    /// The draws as a read-out.
    ///
    /// # Errors
    ///
    /// As [`ShotEstimate::from_effective_draws`]: fewer than half a draw.
    pub fn read_out(&self) -> Result<ShotEstimate<R>, QuantumError> {
        ShotEstimate::from_effective_draws(self.probability, self.draws)
    }
}
