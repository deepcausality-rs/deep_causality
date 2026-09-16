/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the virtual epilepsy surgery planner: the connectome, the Kuramoto dynamics,
//! and the synchronisation measure.
//!
//! The connectome is a [`Graph`] whose payload is one [`RegionState`] per brain region, and the
//! dynamics are a single [`CoMonad::extend`]. `extend` focuses the graph on each region in turn and
//! hands the closure that focused view, so the closure reads its own phase, asks the graph for its
//! neighbours, and returns the region's next phase. The wiring stays in the graph, which is what
//! makes a resection a change to the graph alone.

use crate::FloatType;
use deep_causality_algebra::Real;
use deep_causality_haft::{CoMonad, Foldable};
use deep_causality_num::{Zero, lift, lift_count};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphWitness};

// =============================================================================
// Dynamics
// =============================================================================

/// Coupling strength `K` of the Kuramoto model, in rad/s. The per-neighbour coupling is `K/N`, so
/// a region's pull on the network grows with how many regions it touches. At this value the hub
/// drives the network into synchrony, and each other region on its own leaves it scattered.
pub const COUPLING_STRENGTH: f64 = 6.0;

/// The natural frequency the regions are centred on, in rad/s, and the total spread across them.
/// A narrow spread is what lets a strong hub capture the network.
pub const BASE_FREQUENCY: f64 = 10.0;
pub const FREQUENCY_SPREAD: f64 = 1.0;

/// Integration step in seconds, and how many steps a simulation runs. Thirty seconds of model
/// time is long enough for the network to settle into its steady state.
pub const TIME_STEP_S: f64 = 0.01;
pub const SIMULATION_STEPS: usize = 3000;

/// Synchronisation above this level counts as a seizure. The order parameter runs from 0 for
/// scattered phases to 1 for a network locked in step.
pub const SEIZURE_THRESHOLD: f64 = 0.80;

/// One brain region: an oscillator with a phase and the frequency it runs at when left alone.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RegionState {
    pub phase: FloatType,
    pub natural_frequency: FloatType,
}

/// `Graph` asks its payload for an additive identity so it can size and initialise the backing
/// tensor. A region at the zero state sits at phase zero and runs at zero frequency, and addition
/// is componentwise, which is what makes that identity lawful.
impl core::ops::Add for RegionState {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            phase: self.phase + rhs.phase,
            natural_frequency: self.natural_frequency + rhs.natural_frequency,
        }
    }
}

impl Zero for RegionState {
    fn zero() -> Self {
        let zero = lift::<FloatType>(0.0);
        Self {
            phase: zero,
            natural_frequency: zero,
        }
    }

    fn is_zero(&self) -> bool {
        let zero = lift::<FloatType>(0.0);
        self.phase == zero && self.natural_frequency == zero
    }
}

/// The connectome: regions as vertices, white-matter connections as edges, and one oscillator
/// state per region as the payload.
pub type Connectome = Graph<RegionState>;

// =============================================================================
// Construction
// =============================================================================

/// Builds the connectome, leaving `resected` disconnected when one is named.
///
/// Region 0 is the seizure focus: it connects to every other region, which is the hub topology
/// that drives pathological synchrony. The remaining regions form a local chain. A resection
/// removes a region's connections, so the region stays in the graph with its own dynamics and
/// stops influencing the network.
pub fn build_connectome(
    regions: usize,
    resected: Option<usize>,
) -> Result<Connectome, Box<dyn std::error::Error>> {
    let spread = lift::<FloatType>(FREQUENCY_SPREAD);
    let base = lift::<FloatType>(BASE_FREQUENCY);
    let half = lift::<FloatType>(0.5);
    let span = lift_count::<FloatType>(regions as u64 - 1);

    // A deterministic phase fan and a linear frequency ramp, so every run starts identically.
    let states: Vec<RegionState> = (0..regions)
        .map(|i| {
            let index = lift_count::<FloatType>(i as u64);
            RegionState {
                phase: index * lift::<FloatType>(0.7) % (lift::<FloatType>(2.0) * FloatType::pi()),
                natural_frequency: base + spread * (index / span - half),
            }
        })
        .collect();

    let payload = CausalTensor::new(states, vec![regions])?;
    let mut graph = Graph::new(regions, payload, 0)?;

    let skip = |a: usize, b: usize| resected.is_some_and(|t| t == a || t == b);

    for region in 1..regions {
        if !skip(0, region) {
            graph.add_edge(0, region)?;
        }
    }
    for region in 1..regions - 1 {
        if !skip(region, region + 1) {
            graph.add_edge(region, region + 1)?;
        }
    }

    Ok(graph)
}

// =============================================================================
// Dynamics
// =============================================================================

/// One Kuramoto step over the whole connectome.
///
/// `dθᵢ/dt = ωᵢ + (K/N)·Σⱼ sin(θⱼ − θᵢ)` summed over region `i`'s neighbours. `extend` supplies the
/// cursor, the payload and the adjacency in one focused view, so the coupling sum reads the
/// neighbours straight off the graph.
pub fn kuramoto_step(brain: &Connectome, regions: usize) -> Connectome {
    let dt = lift::<FloatType>(TIME_STEP_S);
    let gain = lift::<FloatType>(COUPLING_STRENGTH) / lift_count::<FloatType>(regions as u64);
    let zero = lift::<FloatType>(0.0);

    GraphWitness::extend(brain, |view| {
        let i = view.cursor();
        let states = view.data().as_slice();
        let here = states[i];

        let coupling = match view.neighbors(i) {
            Ok(neighbours) => neighbours.iter().fold(zero, |sum, &j| {
                sum + Real::sin(states[j].phase - here.phase)
            }),
            Err(_) => zero,
        };

        RegionState {
            phase: wrap_phase(here.phase + dt * (here.natural_frequency + gain * coupling)),
            natural_frequency: here.natural_frequency,
        }
    })
}

/// Folds a phase back into `[0, 2π)`.
///
/// A phase is an angle, so this is the model's own arithmetic. It also keeps the magnitude small,
/// which is what lets a narrow scalar such as `BFloat16` resolve a step of `dt·ω`: over three
/// thousand steps an unwrapped phase reaches several hundred radians, and two decimal digits of
/// significand resolve increments only near the leading digit there.
fn wrap_phase(phase: FloatType) -> FloatType {
    let turn = lift::<FloatType>(2.0) * FloatType::pi();
    let mut wrapped = phase;
    while wrapped >= turn {
        wrapped -= turn;
    }
    let zero = lift::<FloatType>(0.0);
    while wrapped < zero {
        wrapped += turn;
    }
    wrapped
}

/// Runs the connectome forward and returns its synchronisation.
pub fn simulate(brain: &Connectome, regions: usize) -> FloatType {
    let mut state = brain.clone();
    for _ in 0..SIMULATION_STEPS {
        state = kuramoto_step(&state, regions);
    }
    synchronisation(&state)
}

/// The Kuramoto order parameter `|⟨e^{iθ}⟩|` over the regions still wired into the network.
///
/// Two categorical steps carry it. `extend` asks each region for its phasor, reporting `None` for
/// a region a resection left unwired, and `fold` sums the phasors that came back. A resected
/// region keeps oscillating on its own, and leaving it out of the average is what makes the
/// measure describe the network that remains.
pub fn synchronisation(brain: &Connectome) -> FloatType {
    let zero = lift::<FloatType>(0.0);

    let phasors = GraphWitness::extend(brain, |view| {
        let i = view.cursor();
        let phase = view.data().as_slice()[i].phase;
        let wired = view.neighbors(i).map(|n| !n.is_empty()).unwrap_or(false);
        if wired {
            Some((Real::cos(phase), Real::sin(phase)))
        } else {
            None
        }
    });

    let (sum_cos, sum_sin, counted) =
        GraphWitness::fold(phasors, (zero, zero, 0u64), |acc, phasor| match phasor {
            Some((c, s)) => (acc.0 + c, acc.1 + s, acc.2 + 1),
            None => acc,
        });

    if counted == 0 {
        return zero;
    }
    Real::sqrt(sum_cos * sum_cos + sum_sin * sum_sin) / lift_count::<FloatType>(counted)
}

/// Whether a synchronisation reading counts as a seizure.
pub fn is_seizing(sync: FloatType) -> bool {
    sync > lift::<FloatType>(SEIZURE_THRESHOLD)
}
