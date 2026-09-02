/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shot statistics at the pipeline's scalar.
//!
//! The shipped bridges `shots_to_qubit_bernoulli` and `shots_to_observable` accumulate in `f64`
//! and return `Uncertain<_>`. They keep their signatures; this is the scalar-generic estimator
//! beside them.
//!
//! # The bound is `RealField`, and the design note's row said `Real`
//!
//! §6.4 of the design note has this surface at `Real + FromPrimitive`, on the reasoning that
//! `sqrt`, `log2` and ratios touch no complex carrier so dual numbers should stay admissible.
//! The premise fails on the first line: `p = k / n` is a ratio, and `Real` in
//! `deep_causality_algebra` is `CommutativeRing + PartialOrd + Neg + …` with no `Div`. Division
//! arrives with `Field`, so the weakest structure that carries a frequency is `RealField`, and
//! the row is corrected here rather than worked around.

use crate::QuantumError;
use crate::types::decision::{Check, CheckItem, CheckReport, Tolerance};
use crate::types::qpu::histogram::ShotHistogram;
use alloc::format;
use alloc::vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// A Bernoulli point estimate from a histogram, with its standard error and the shots behind it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShotEstimate<R> {
    estimate: R,
    standard_error: R,
    shots: u64,
}

impl<R> ShotEstimate<R>
where
    R: RealField + FromPrimitive,
{
    fn from_counts(hits: u64, total: u64) -> Result<Self, QuantumError> {
        if total == 0 {
            return Err(QuantumError::NormalizationError(
                "cannot bridge an empty shot histogram".into(),
            ));
        }
        let n = R::from_u64(total).ok_or_else(|| {
            QuantumError::CalculationError(format!("scalar cannot represent {total} shots"))
        })?;
        let k = R::from_u64(hits).ok_or_else(|| {
            QuantumError::CalculationError(format!("scalar cannot represent {hits} hits"))
        })?;
        let p = k / n;
        let one = R::one();
        let standard_error = (p * (one - p) / n).sqrt();
        Ok(Self {
            estimate: p,
            standard_error,
            shots: total,
        })
    }

    /// The frequency of one outcome.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NormalizationError`] on an empty histogram, matching the shipped bridges.
    pub fn of_outcome<H: ShotHistogram>(hist: &H, outcome: usize) -> Result<Self, QuantumError> {
        Self::from_counts(hist.count(outcome), hist.total())
    }

    /// The frequency of `1` on the `bit_index`-th measured qubit, as `shots_to_qubit_bernoulli`
    /// reads it.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NormalizationError`] on an empty histogram;
    /// [`QuantumError::DimensionMismatch`] on a bit beyond the measured width.
    pub fn of_bit<H: ShotHistogram>(hist: &H, bit_index: usize) -> Result<Self, QuantumError> {
        if bit_index >= hist.num_bits() {
            return Err(QuantumError::DimensionMismatch(format!(
                "bit index {} ≥ measured qubits {}",
                bit_index,
                hist.num_bits()
            )));
        }
        let ones: u64 = hist
            .entries()
            .into_iter()
            .filter(|(outcome, _)| (outcome >> bit_index) & 1 == 1)
            .map(|(_, count)| count)
            .sum();
        Self::from_counts(ones, hist.total())
    }

    /// The point estimate.
    pub fn estimate(&self) -> R {
        self.estimate
    }

    /// `√(p(1−p)/n)`, the shot-noise width.
    pub fn standard_error(&self) -> R {
        self.standard_error
    }

    /// The shots the estimate came from.
    pub fn shots(&self) -> u64 {
        self.shots
    }

    /// The Bhattacharyya distance between two Bernoulli estimates, in bits:
    /// `−log₂(√(pq) + √((1−p)(1−q)))`. Zero for equal estimates, unbounded as they separate.
    pub fn separation_bits(&self, other: &Self) -> R {
        let one = R::one();
        let (p, q) = (self.estimate, other.estimate);
        let bc = (p * q).sqrt() + ((one - p) * (one - q)).sqrt();
        -bc.log2()
    }
}

impl<R> ShotEstimate<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// Whether the estimate reaches `spec` from below, as a margin over shots.
    ///
    /// The measured quantity is the shortfall `spec − estimate`, the threshold is the shot-noise
    /// width from [`Tolerance::shot_noise`], and the examined count is the shots. A shortfall
    /// within one standard error accepts; a negative shortfall reads as a negative margin, the
    /// distance above the spec in units of the noise.
    pub fn at_least(&self, spec: R) -> CheckReport<R> {
        self.against(spec - self.estimate)
    }

    /// Whether the estimate stays at or below `spec`, as a margin over shots.
    pub fn at_most(&self, spec: R) -> CheckReport<R> {
        self.against(self.estimate - spec)
    }

    fn against(&self, excess: R) -> CheckReport<R> {
        let width = Tolerance::<R>::shot_noise()
            .shot_noise_width(self.estimate, self.shots)
            .expect("the shot-noise member answers the read-out form");
        let examined = usize::try_from(self.shots).unwrap_or(usize::MAX);
        CheckReport::new(vec![Check::new(CheckItem::Whole, excess, width)], examined)
    }
}
