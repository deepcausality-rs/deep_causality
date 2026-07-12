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
use std::collections::BTreeMap;

/// A classical measurement histogram: outcome bitstrings (packed LSB-first over
/// the circuit's measured qubits) to shot counts. Never exposes amplitudes.
pub trait ShotHistogram {
    /// The total number of shots recorded.
    fn total(&self) -> u64;

    /// The number of measured qubits (outcome bitstring width).
    fn num_bits(&self) -> usize;

    /// The shot count for a given outcome (a bitstring packed as a `usize`).
    fn count(&self, outcome: usize) -> u64;

    /// The non-zero `(outcome, count)` entries, ascending by outcome.
    fn entries(&self) -> Vec<(usize, u64)>;
}

/// The concrete outcome-count histogram returned by the in-process simulator.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CountHistogram {
    counts: BTreeMap<usize, u64>,
    total: u64,
    num_bits: usize,
}

impl CountHistogram {
    /// An empty histogram over `num_bits` measured qubits.
    pub fn new(num_bits: usize) -> Self {
        Self {
            counts: BTreeMap::new(),
            total: 0,
            num_bits,
        }
    }

    /// Records one shot with the given outcome.
    pub fn record(&mut self, outcome: usize) {
        *self.counts.entry(outcome).or_insert(0) += 1;
        self.total += 1;
    }

    /// Records `n` shots of the given outcome at once.
    pub fn record_n(&mut self, outcome: usize, n: u64) {
        if n == 0 {
            return;
        }
        *self.counts.entry(outcome).or_insert(0) += n;
        self.total += n;
    }
}

impl ShotHistogram for CountHistogram {
    fn total(&self) -> u64 {
        self.total
    }

    fn num_bits(&self) -> usize {
        self.num_bits
    }

    fn count(&self, outcome: usize) -> u64 {
        self.counts.get(&outcome).copied().unwrap_or(0)
    }

    fn entries(&self) -> Vec<(usize, u64)> {
        self.counts.iter().map(|(&o, &c)| (o, c)).collect()
    }
}

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
