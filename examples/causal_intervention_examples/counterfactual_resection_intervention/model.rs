/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Connectome construction, Kuramoto simulation, and chain constructors.

use crate::model_types::{
    COUPLING_STRENGTH, Connectome, DT, FloatType, SEIZURE_THRESHOLD, SeizureResult, TIME_STEPS,
};
use deep_causality_core::{EffectValue, PropagatingEffect};
use std::f64::consts::PI;

impl Connectome {
    /// Hub-and-spoke pattern. Region 0 connects to every other region,
    /// and the rest form a sparse chain. The hub drives synchronisation,
    /// which is the canonical "seizure focus" connectivity.
    pub fn hub_and_spoke(n: usize) -> Self {
        let mut adj = vec![Vec::new(); n];
        for i in 1..n {
            adj[0].push(i);
            adj[i].push(0);
        }
        for i in 1..n - 1 {
            adj[i].push(i + 1);
            adj[i + 1].push(i);
        }
        // Tuned so the hub-driven coupling barely overcomes individual
        // frequency differences in the factual graph (seizure). With the
        // hub disconnected, the remaining periphery's coupling is too
        // weak to maintain synchronisation (stable).
        let intrinsic_freq: Vec<FloatType> = (0..n).map(|i| 1.0 + 0.05 * i as FloatType).collect();
        let initial_phase: Vec<FloatType> = (0..n)
            .map(|i| (i as FloatType * 0.5) % (2.0 * PI))
            .collect();
        Self {
            adj,
            intrinsic_freq,
            initial_phase,
        }
    }

    /// Return a copy of the connectome with `target` resected. All edges
    /// incident to `target` are removed; the node remains as an isolated
    /// vertex. This matches the original surgical-resection semantics.
    pub fn resected(&self, target: usize) -> Self {
        let new_adj: Vec<Vec<usize>> = self
            .adj
            .iter()
            .enumerate()
            .map(|(u, neighbors)| {
                if u == target {
                    Vec::new()
                } else {
                    neighbors.iter().copied().filter(|&v| v != target).collect()
                }
            })
            .collect();
        Self {
            adj: new_adj,
            intrinsic_freq: self.intrinsic_freq.clone(),
            initial_phase: self.initial_phase.clone(),
        }
    }
}

/// Run the Kuramoto simulation against the connectome carried in the
/// value channel and produce the post-simulation synchronisation level.
pub fn simulate_seizure(
    value: EffectValue<Connectome>,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<SeizureResult> {
    let connectome = match value.into_value() {
        Some(c) => c,
        None => return PropagatingEffect::pure(SeizureResult::default()),
    };
    let n = connectome.adj.len();
    let mut phases = connectome.initial_phase.clone();
    let freqs = &connectome.intrinsic_freq;

    let mut final_sync: FloatType = 0.0;
    for _ in 0..TIME_STEPS {
        let mut next = phases.clone();
        for i in 0..n {
            let mut coupling: FloatType = 0.0;
            for &j in &connectome.adj[i] {
                coupling += (phases[j] - phases[i]).sin();
            }
            let d_theta = freqs[i] + (COUPLING_STRENGTH / n as FloatType) * coupling;
            next[i] += d_theta * DT;
        }
        phases = next;

        let sum_cos: FloatType = phases.iter().map(|p| p.cos()).sum();
        let sum_sin: FloatType = phases.iter().map(|p| p.sin()).sum();
        final_sync = (sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / n as FloatType;
    }

    PropagatingEffect::pure(SeizureResult {
        final_sync,
        seizing: final_sync > SEIZURE_THRESHOLD,
    })
}

/// Build the chain once with the factual (full) connectome as the entry
/// value. Counterfactuals intervene on this entry before the simulate stage.
pub fn build_chain(factual: Connectome) -> PropagatingEffect<Connectome> {
    PropagatingEffect::pure(factual)
}
