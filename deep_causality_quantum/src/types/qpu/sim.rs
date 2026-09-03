/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! An in-process, deterministic state-vector simulator satisfying the
//! [`QpuSampler`] seam (R2) — the hardware-free default so Phase-3 and the
//! examples can exercise the emergent path without any network/async/vendor
//! dependency. Given a fixed seed it reproduces the same histogram exactly.

use crate::QuantumError;
use crate::types::qpu::circuit::{GateOp, QuantumCircuit};
use crate::types::qpu::histogram::CountHistogram;
use crate::types::qpu::prng::SplitMix64;
use crate::types::qpu::sampler::QpuSampler;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;

/// The calibration surfaced to the context channel by `qpu_effect`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimCalibration {
    /// A human-readable device label.
    pub name: String,
    /// The simulator's PRNG seed (part of what makes a run reproducible).
    pub seed: u64,
}

/// A deterministic dense state-vector simulator. Amplitudes never leave the
/// simulator — `sample` returns only a classical [`CountHistogram`].
#[derive(Debug, Clone)]
pub struct SimQpu {
    seed: u64,
    name: String,
}

impl SimQpu {
    /// A simulator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            name: "deep_causality_quantum::SimQpu".to_string(),
        }
    }

    /// A simulator with a seed and a device label.
    pub fn with_name(seed: u64, name: impl Into<String>) -> Self {
        Self {
            seed,
            name: name.into(),
        }
    }
}

// A complex amplitude as a plain pair (avoids leaning on operator ergonomics).
#[derive(Clone, Copy)]
struct C {
    re: f64,
    im: f64,
}

impl C {
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    fn add(self, o: C) -> C {
        C::new(self.re + o.re, self.im + o.im)
    }
    fn mul(self, o: C) -> C {
        C::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }
    fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}

fn apply_single(state: &mut [C], q: usize, m: [C; 4]) {
    let bit = 1usize << q;
    let n = state.len();
    let mut i = 0;
    while i < n {
        if i & bit == 0 {
            let j = i | bit;
            let a0 = state[i];
            let a1 = state[j];
            state[i] = m[0].mul(a0).add(m[1].mul(a1));
            state[j] = m[2].mul(a0).add(m[3].mul(a1));
        }
        i += 1;
    }
}

fn apply_cnot(state: &mut [C], control: usize, target: usize) {
    let cb = 1usize << control;
    let tb = 1usize << target;
    let n = state.len();
    for i in 0..n {
        // Swap the pair differing in the target bit exactly once (when target=0).
        if i & cb != 0 && i & tb == 0 {
            state.swap(i, i | tb);
        }
    }
}

// Every diagonal gate in the alphabet is one operation at a different arity and
// a different phase: multiply the amplitude by `phase` on exactly those basis
// states with all of `qubits` set, and leave every other amplitude alone. `Z`,
// `S`, `S†`, `T` and `T†` are the one-qubit cases; `CZ`, `CS†`, `CCZ` and
// `C^{m-1}Z` are the multi-qubit ones. Folding the qubit list into a mask makes
// the arity a runtime value, so `C^{m-1}Z` needs no kernel of its own.
//
// A mask of zero would phase the whole register, a global phase with no
// observable effect. `QuantumCircuit::new` rejects an empty qubit list, so no
// circuit reaches here with one.
fn apply_diagonal_phase(state: &mut [C], qubits: &[usize], phase: C) {
    let mut mask = 0usize;
    for &q in qubits {
        mask |= 1usize << q;
    }
    for (i, a) in state.iter_mut().enumerate() {
        if i & mask == mask {
            *a = phase.mul(*a);
        }
    }
}

impl QpuSampler for SimQpu {
    type Shots = CountHistogram;
    type Calibration = SimCalibration;

    fn sample(&self, circuit: &QuantumCircuit, shots: u64) -> Result<Self::Shots, QuantumError> {
        let n = circuit.num_qubits();
        if n > 24 {
            return Err(QuantumError::DimensionMismatch(format!(
                "SimQpu caps at 24 qubits (2^24 amplitudes); circuit has {}",
                n
            )));
        }
        let dim = 1usize << n;
        let mut state = vec![C::new(0.0, 0.0); dim];
        state[0] = C::new(1.0, 0.0);

        let s = 1.0 / core::f64::consts::SQRT_2;
        let pi4 = core::f64::consts::FRAC_PI_4;
        // e^{iπ/4}, the T phase. T† is its conjugate.
        let (t_re, t_im) = (pi4.cos(), pi4.sin());
        let minus_one = C::new(-1.0, 0.0);
        for op in circuit.ops() {
            // Exhaustive over all GateOp variants — no catch-all, so a new gate
            // must be handled explicitly rather than silently mis-applied.
            match op {
                GateOp::H(q) => apply_single(
                    &mut state,
                    *q,
                    [
                        C::new(s, 0.0),
                        C::new(s, 0.0),
                        C::new(s, 0.0),
                        C::new(-s, 0.0),
                    ],
                ),
                GateOp::X(q) => apply_single(
                    &mut state,
                    *q,
                    [
                        C::new(0.0, 0.0),
                        C::new(1.0, 0.0),
                        C::new(1.0, 0.0),
                        C::new(0.0, 0.0),
                    ],
                ),
                GateOp::Y(q) => apply_single(
                    &mut state,
                    *q,
                    [
                        C::new(0.0, 0.0),
                        C::new(0.0, -1.0),
                        C::new(0.0, 1.0),
                        C::new(0.0, 0.0),
                    ],
                ),
                // The diagonal family, one kernel and one phase each.
                GateOp::Z(q) => apply_diagonal_phase(&mut state, &[*q], minus_one),
                GateOp::S(q) => apply_diagonal_phase(&mut state, &[*q], C::new(0.0, 1.0)),
                GateOp::Sdg(q) => apply_diagonal_phase(&mut state, &[*q], C::new(0.0, -1.0)),
                GateOp::T(q) => apply_diagonal_phase(&mut state, &[*q], C::new(t_re, t_im)),
                GateOp::Tdg(q) => apply_diagonal_phase(&mut state, &[*q], C::new(t_re, -t_im)),
                GateOp::Cnot { control, target } => apply_cnot(&mut state, *control, *target),
                GateOp::Cz { control, target } => {
                    apply_diagonal_phase(&mut state, &[*control, *target], minus_one)
                }
                GateOp::Csdg { control, target } => {
                    apply_diagonal_phase(&mut state, &[*control, *target], C::new(0.0, -1.0))
                }
                GateOp::Ccz { q0, q1, q2 } => {
                    apply_diagonal_phase(&mut state, &[*q0, *q1, *q2], minus_one)
                }
                GateOp::Cmz { qubits } => apply_diagonal_phase(&mut state, qubits, minus_one),
            }
        }

        let measure = circuit.measure();
        let num_bits = measure.len();
        let mut hist = CountHistogram::new(num_bits)?;
        if shots == 0 {
            return Ok(hist);
        }

        // Marginal distribution over the measured qubits, packed LSB-first in
        // measurement order.
        let num_outcomes = 1usize << num_bits;
        let mut probs = vec![0.0f64; num_outcomes];
        for (f, amp) in state.iter().enumerate() {
            let mut outcome = 0usize;
            for (k, &mq) in measure.iter().enumerate() {
                if (f >> mq) & 1 == 1 {
                    outcome |= 1 << k;
                }
            }
            probs[outcome] += amp.norm_sq();
        }

        // Cumulative distribution for inverse-CDF sampling.
        let mut cum = vec![0.0f64; num_outcomes];
        let mut acc = 0.0;
        for (i, p) in probs.iter().enumerate() {
            acc += *p;
            cum[i] = acc;
        }
        let total = acc.max(f64::MIN_POSITIVE);

        let mut rng = SplitMix64::new(self.seed);
        for _ in 0..shots {
            let u = rng.next_f64() * total;
            // First index whose cumulative mass exceeds u. `cum` is non-decreasing,
            // so binary-search it (same boundary as the old linear "first u < c"
            // scan), clamping the degenerate all-mass-below-u case to the last bin.
            let outcome = cum.partition_point(|&c| c <= u).min(num_outcomes - 1);
            hist.record(outcome)?;
        }

        Ok(hist)
    }

    fn calibration(&self) -> SimCalibration {
        SimCalibration {
            name: self.name.clone(),
            seed: self.seed,
        }
    }
}
