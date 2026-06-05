/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Connectome construction, Kuramoto simulation, and chain constructors.

use crate::model_types::{
    COUPLING_STRENGTH, Connectome, DT, FloatType, SEIZURE_THRESHOLD, SeizureResult, TIME_STEPS,
};
use deep_causality_calculus::{EndoArrow, Euler};
use std::f64::consts::PI;
use std::ops::{Add, Mul};

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

/// Run the Kuramoto simulation against the connectome carried by the flow
/// and produce the post-simulation synchronisation level. `CausalFlow::map`
/// supplies the connectome, so the stage reads as a plain transform.
pub fn simulate_seizure(connectome: Connectome) -> SeizureResult {
    let n = connectome.adj.len();
    let phases = Phases(connectome.initial_phase.clone());
    let adj = connectome.adj;
    let freqs = connectome.intrinsic_freq;

    // Kuramoto rate field dθ_i/dt = ω_i + (K/N)·Σ_j sin(θ_j − θ_i), expressed as an Euler
    // endo-arrow. The hand-rolled phase loop becomes iterate_n. Substituting Rk4 raises the
    // integration order with no change to this rate field.
    let step = Euler::new(DT, move |p: &Phases| {
        let d = (0..n)
            .map(|i| {
                let coupling: FloatType = adj[i].iter().map(|&j| (p.0[j] - p.0[i]).sin()).sum();
                freqs[i] + (COUPLING_STRENGTH / n as FloatType) * coupling
            })
            .collect();
        Phases(d)
    });

    // March the oscillators; the seizure verdict reads the synchronisation of the final state.
    let final_phases = step.iterate_n(phases, TIME_STEPS);

    let sum_cos: FloatType = final_phases.0.iter().map(|p| p.cos()).sum();
    let sum_sin: FloatType = final_phases.0.iter().map(|p| p.sin()).sum();
    let final_sync = (sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / n as FloatType;

    SeizureResult {
        final_sync,
        seizing: final_sync > SEIZURE_THRESHOLD,
    }
}

/// Kuramoto phase vector, the integrator state. `Euler` and `Rk4` need a module-valued state
/// (`Add` plus scalar `Mul`), which `Vec<FloatType>` lacks, so the phases ride in this newtype.
#[derive(Clone)]
struct Phases(Vec<FloatType>);

impl Add for Phases {
    type Output = Phases;
    fn add(self, rhs: Phases) -> Phases {
        Phases(self.0.iter().zip(rhs.0).map(|(a, b)| a + b).collect())
    }
}

impl Mul<FloatType> for Phases {
    type Output = Phases;
    fn mul(self, s: FloatType) -> Phases {
        Phases(self.0.iter().map(|x| x * s).collect())
    }
}
