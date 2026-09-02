/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Classical outcome counts: the one shape both sampling paths report.
//!
//! [`ShotHistogram`] and [`CountHistogram`] are plain data with no dependency of their own, and
//! the default-build Born sampler returns them, so they are compiled in every build. The
//! `QpuSampler` seam that draws them from a circuit stays behind the `qpu` feature; this is the
//! same split the circuit data types received when the Haruna layer began emitting them.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

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
