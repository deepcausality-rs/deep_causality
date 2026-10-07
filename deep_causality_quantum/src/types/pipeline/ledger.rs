/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use alloc::format;
use deep_causality_algebra::RealField;
use deep_causality_num::NaturalNumber;

/// The threaded state of a run: what was spent on the device, and what was only computed.
///
/// Device time is the scarce resource. Context is read-only and cannot accumulate, the log is a
/// record rather than a running total, and the value is the answer rather than the meter, so
/// state is the only channel both writable and threaded, and this is what rides it.
///
/// Counts are ℕ on `N: NaturalNumber`, whose width a program names once with `NumberType`;
/// widening it buys headroom and moves no threshold. Real quantities are on `R`. `Copy`, with no
/// `Vec` and no `String`, and a hand-written `Default` from the two zeros, because the causal
/// monad requires `State: Default` and a derived one would demand `R: Default`.
///
/// # Three invariants
///
/// `observe`, `baseline` and `observe_experiment` are the only stages that touch `shots`,
/// `experiments` and `device_time`, and the only ones that draw down the budget's remainder,
/// which the control stage sets from named evidence under `qpu`; `predict` and `predict_with`
/// touch `predictions` and nothing on the device side. `fork` is the pipeline's, above core, by cloning. Forked ledgers are compared,
/// never joined under ∇: at a counterfactual fork exactly one branch was factual, and a monoid
/// that summed them would typecheck and be wrong.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ledger<R, N> {
    shots: N,
    experiments: N,
    predictions: N,
    device_time: R,
    cost: R,
    bits: R,
    #[cfg(feature = "qpu")]
    remaining: Option<N>,
}

impl<R: RealField, N: NaturalNumber> Ledger<R, N> {
    /// The empty ledger.
    pub fn new() -> Self {
        Self {
            shots: N::zero(),
            experiments: N::zero(),
            predictions: N::zero(),
            device_time: R::zero(),
            cost: R::zero(),
            bits: R::zero(),
            #[cfg(feature = "qpu")]
            remaining: None,
        }
    }

    /// The same ledger with `shots` as the budget every later observation draws from. Only the
    /// control stage calls this, from the evidence a configuration names under `qpu`.
    ///
    /// The budget belongs to the ledger: `fork` copies the ledger into every world, so each world
    /// draws on its own copy of the remainder, and the budget bounds the shots along the root and
    /// any one world, not their sum across worlds.
    #[cfg(feature = "qpu")]
    pub(crate) fn budgeted(self, shots: N) -> Self {
        Self {
            remaining: Some(shots),
            ..self
        }
    }

    /// Shots taken on the device.
    pub fn shots(&self) -> N {
        self.shots
    }

    /// Experiments executed on hardware.
    pub fn experiments(&self) -> N {
        self.experiments
    }

    /// Model evaluations: tracked, never billed.
    pub fn predictions(&self) -> N {
        self.predictions
    }

    /// Accumulated device time.
    pub fn device_time(&self) -> R {
        self.device_time
    }

    /// The cost the design objective minimised.
    pub fn cost(&self) -> R {
        self.cost
    }

    /// The separation achieved so far, in bits.
    pub fn bits(&self) -> R {
        self.bits
    }

    /// The shots the evidence budget still allows on this ledger, or `None` when the run names no
    /// budget. After `fork` every world holds its own copy of the remainder and draws on it alone.
    #[cfg(feature = "qpu")]
    pub fn remaining(&self) -> Option<N> {
        self.remaining
    }

    /// The remainder of `budget` after drawing `request` from it, in checked ℕ arithmetic.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] naming the shortfall when `request` exceeds `budget`:
    /// `checked_difference` returned `None`, and `monus` says by how much.
    pub fn draw_down(budget: N, request: N) -> Result<N, QuantumError>
    where
        N: core::fmt::Debug,
    {
        budget.checked_difference(request).ok_or_else(|| {
            QuantumError::CalculationError(format!(
                "shot budget overdrawn: requested {:?} against {:?}, shortfall {:?}",
                request,
                budget,
                request.monus(budget)
            ))
        })
    }

    /// The ledger after one hardware observation of `shots` shots taking `device_time`, drawn
    /// down from the budget when there is one (under `qpu`). Only the observing stages call this.
    ///
    /// # Errors
    ///
    /// The overdraw error of [`draw_down`](Self::draw_down) when `shots` exceeds the remaining
    /// budget, and [`QuantumError::CalculationError`] when a count overflows the width.
    pub(crate) fn observed(self, shots: N, device_time: R) -> Result<Self, QuantumError>
    where
        N: core::fmt::Debug,
    {
        #[cfg(feature = "qpu")]
        let remaining = self
            .remaining
            .map(|budget| Self::draw_down(budget, shots))
            .transpose()?;
        let total = self.shots.checked_add(shots).ok_or_else(|| {
            QuantumError::CalculationError(format!(
                "shot count overflows the width: {:?} + {:?}",
                self.shots, shots
            ))
        })?;
        let experiments = self.experiments.succ().ok_or_else(|| {
            QuantumError::CalculationError("experiment count overflows the width".into())
        })?;
        Ok(Self {
            shots: total,
            experiments,
            device_time: self.device_time + device_time,
            #[cfg(feature = "qpu")]
            remaining,
            ..self
        })
    }

    /// The ledger after one model evaluation. Only `predict` and `predict_with` call this.
    pub(crate) fn predicted(self) -> Result<Self, QuantumError> {
        let predictions = self.predictions.succ().ok_or_else(|| {
            QuantumError::CalculationError("prediction count overflows the width".into())
        })?;
        Ok(Self {
            predictions,
            ..self
        })
    }

    /// The ledger after a plan's cost is committed.
    pub(crate) fn costed(self, cost: R) -> Self {
        Self {
            cost: self.cost + cost,
            ..self
        }
    }

    /// The ledger after a separation is achieved.
    pub(crate) fn separated(self, bits: R) -> Self {
        Self {
            bits: self.bits + bits,
            ..self
        }
    }
}

impl<R: RealField, N: NaturalNumber> Default for Ledger<R, N> {
    fn default() -> Self {
        Self::new()
    }
}
