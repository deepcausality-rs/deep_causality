/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The classical-shot sampler seam (R2). `QpuSampler` is used only as a generic
//! bound `S: QpuSampler` (never `dyn`); its associated `Shots` type is bounded
//! by [`ShotHistogram`], which exposes a classical outcome-count map — never
//! amplitudes — pinning the Kleisli/coherence boundary at the type level.

use crate::QuantumError;
use crate::types::qpu::circuit::QuantumCircuit;
use crate::types::qpu::histogram::ShotHistogram;

/// The generic sampler seam. Implementations return measurement shots as
/// classical [`ShotHistogram`] data at the Kleisli cut; no concrete vendor
/// adapter is shipped by this crate. Used only as a bound `S: QpuSampler`.
pub trait QpuSampler {
    /// The classical shot histogram this sampler returns.
    type Shots: ShotHistogram;

    /// The device calibration / topology metadata surfaced to the context
    /// channel by `qpu_effect`.
    type Calibration;

    /// Samples `shots` executions of `circuit`, returning the classical outcome
    /// histogram or a typed failure. Deterministic implementations reproduce the
    /// same histogram for the same input.
    fn sample(&self, circuit: &QuantumCircuit, shots: u64) -> Result<Self::Shots, QuantumError>;

    /// The device calibration surfaced at the Kleisli boundary.
    fn calibration(&self) -> Self::Calibration;
}
