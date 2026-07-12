/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Bridges from a classical [`ShotHistogram`] to `deep_causality_uncertain`
//! values, and the causaloid lift `qpu_effect` that routes a physical-QPU call
//! into the arity-5 causal monad (R2). No new value substance is introduced;
//! measurement statistics surface as `Uncertain<_>`, and a job failure routes to
//! the error channel with the value absent.

use crate::QuantumError;
use crate::types::qpu::circuit::QuantumCircuit;
use crate::types::qpu::sampler::{QpuSampler, ShotHistogram};
use core::fmt::Debug;
use deep_causality_core::{
    CausalEffectPropagationProcess, CausalityError, EffectLog, PropagatingEffect,
};
use deep_causality_haft::LogAddEntry;
use deep_causality_uncertain::Uncertain;

/// The requested-parameter summary routed to the STATE channel by
/// [`qpu_effect`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QpuParams {
    pub num_qubits: usize,
    pub num_ops: usize,
    pub num_measured: usize,
    pub shots: u64,
}

/// A per-qubit `Uncertain<bool>` from the histogram: the Bernoulli distribution
/// whose success probability is the measured frequency of `1` on the
/// `bit_index`-th measured qubit (LSB-first in measurement order).
///
/// # Errors
/// Returns a typed [`QuantumError`] on an empty histogram or an out-of-range
/// bit index.
pub fn shots_to_qubit_bernoulli<H: ShotHistogram>(
    hist: &H,
    bit_index: usize,
) -> Result<Uncertain<bool>, QuantumError> {
    let total = hist.total();
    if total == 0 {
        return Err(QuantumError::NormalizationError(
            "cannot bridge an empty shot histogram".into(),
        ));
    }
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
    let p = ones as f64 / total as f64;
    Ok(Uncertain::bernoulli(p))
}

/// An observable `Uncertain<f64>` from the histogram: each outcome is mapped to
/// a real value by `value_of`, and the empirical distribution over the shots is
/// summarized via `Uncertain::from_samples`. `O(total shots)` in the expansion.
///
/// # Errors
/// Returns a typed [`QuantumError`] on an empty histogram.
pub fn shots_to_observable<H, F>(hist: &H, value_of: F) -> Result<Uncertain<f64>, QuantumError>
where
    H: ShotHistogram,
    F: Fn(usize) -> f64,
{
    let total = hist.total();
    if total == 0 {
        return Err(QuantumError::NormalizationError(
            "cannot bridge an empty shot histogram".into(),
        ));
    }
    let mut samples: Vec<f64> = Vec::with_capacity(total as usize);
    for (outcome, count) in hist.entries() {
        let v = value_of(outcome);
        for _ in 0..count {
            samples.push(v);
        }
    }
    Ok(Uncertain::from_samples(&samples))
}

/// Lifts a physical-QPU call into a causaloid `f` at the Kleisli boundary: on
/// success the shot histogram rides the VALUE channel, the requested parameters
/// the STATE channel, the device calibration the CONTEXT channel, and the
/// provenance the LOG channel; a job failure rides the ERROR channel with the
/// value absent. Generic over `S: QpuSampler` (no `dyn`).
pub fn qpu_effect<S>(
    sampler: &S,
    circuit: &QuantumCircuit,
    shots: u64,
) -> CausalEffectPropagationProcess<S::Shots, QpuParams, S::Calibration, CausalityError, EffectLog>
where
    S: QpuSampler,
    S::Shots: Default + Clone + Debug,
    S::Calibration: Clone + Debug,
{
    let params = QpuParams {
        num_qubits: circuit.num_qubits(),
        num_ops: circuit.ops().len(),
        num_measured: circuit.measure().len(),
        shots,
    };

    match sampler.sample(circuit, shots) {
        Ok(hist) => {
            let calibration = sampler.calibration();
            let mut provenance = EffectLog::from(format!(
                "qpu: sampled {} shot(s) of a {}-qubit circuit with {} gate(s)",
                shots,
                circuit.num_qubits(),
                circuit.ops().len()
            ));
            provenance.add_entry(&format!("qpu: calibration = {:?}", calibration));
            let effect: PropagatingEffect<S::Shots> =
                PropagatingEffect::from_value_with_log(hist, provenance);
            CausalEffectPropagationProcess::with_state(effect, params, Some(calibration))
        }
        Err(e) => CausalEffectPropagationProcess::from_error(CausalityError::from(e)),
    }
}
