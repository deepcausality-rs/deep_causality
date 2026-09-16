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
use deep_causality_num::{Zero, const_scalar_from_float, const_scalar_from_int, lift_count};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphWitness, TopologyError};

// =============================================================================
// Dynamics
// =============================================================================

/// The small numbers the dynamics are written with.
pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
const HALF: FloatType = const_scalar_from_float!(FloatType, 0.5);
/// The phase each region starts at, as a multiple of its index, in radians.
const PHASE_FAN: FloatType = const_scalar_from_float!(FloatType, 0.7);

/// Coupling strength `K` of the Kuramoto model, in rad/s. The per-neighbour coupling is `K/N`, so
/// a region's pull on the network grows with how many regions it touches. At this value the hub
/// drives the network into synchrony, and each other region on its own leaves it scattered.
pub const COUPLING_STRENGTH: FloatType = const_scalar_from_int!(FloatType, 6);

/// The natural frequency the regions are centred on, in rad/s, and the total spread across them.
/// A narrow spread is what lets a strong hub capture the network.
pub const BASE_FREQUENCY: FloatType = const_scalar_from_int!(FloatType, 10);
pub const FREQUENCY_SPREAD: FloatType = const_scalar_from_int!(FloatType, 1);

/// Integration step in seconds, and how many steps a simulation runs. Thirty seconds of model
/// time is long enough for the network to settle into its steady state.
pub const TIME_STEP_S: FloatType = const_scalar_from_float!(FloatType, 0.01);
pub const SIMULATION_STEPS: usize = 3000;

/// Synchronisation above this level counts as a seizure. The order parameter runs from 0 for
/// scattered phases to 1 for a network locked in step.
pub const SEIZURE_THRESHOLD: FloatType = const_scalar_from_float!(FloatType, 0.80);

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
        Self {
            phase: ZERO,
            natural_frequency: ZERO,
        }
    }

    fn is_zero(&self) -> bool {
        self.phase == ZERO && self.natural_frequency == ZERO
    }
}

/// The connectome: regions as vertices, white-matter connections as edges, and one oscillator
/// state per region as the payload.
pub type Connectome = Graph<RegionState>;

// =============================================================================
// Construction
// =============================================================================

/// The fewest regions a connectome holds: a hub and one region for it to drive.
pub const MIN_REGIONS: usize = 2;

/// Builds the connectome, leaving `resected` disconnected when one is named.
///
/// Region 0 is the seizure focus: it connects to every other region, which is the hub topology
/// that drives pathological synchrony. The remaining regions form a local chain. A resection
/// removes a region's connections, so the region stays in the graph with its own dynamics and
/// stops influencing the network.
///
/// The frequency ramp spans the regions from first to last, so the connectome holds at least
/// [`MIN_REGIONS`] of them; fewer is reported as an error.
pub fn build_connectome(
    regions: usize,
    resected: Option<usize>,
) -> Result<Connectome, Box<dyn std::error::Error>> {
    if regions < MIN_REGIONS {
        return Err(TopologyError::GraphError(format!(
            "a connectome holds at least {MIN_REGIONS} regions, {regions} given"
        ))
        .into());
    }
    let span = lift_count::<FloatType>((regions - 1) as u64);

    // A deterministic phase fan and a linear frequency ramp, so every run starts identically.
    let states: Vec<RegionState> = (0..regions)
        .map(|i| {
            let index = lift_count::<FloatType>(i as u64);
            RegionState {
                phase: index * PHASE_FAN % (TWO * FloatType::pi()),
                natural_frequency: BASE_FREQUENCY + FREQUENCY_SPREAD * (index / span - HALF),
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
/// neighbours straight off the graph. `N` is the graph's own vertex count, so the normalisation
/// follows the connectome it is applied to.
pub fn kuramoto_step(brain: &Connectome) -> Connectome {
    let gain = COUPLING_STRENGTH / lift_count::<FloatType>(brain.num_vertices() as u64);

    GraphWitness::extend(brain, |view| {
        let i = view.cursor();
        let states = view.data().as_slice();
        let here = states[i];

        let coupling = match view.neighbors(i) {
            Ok(neighbours) => neighbours.iter().fold(ZERO, |sum, &j| {
                sum + Real::sin(states[j].phase - here.phase)
            }),
            Err(_) => ZERO,
        };

        RegionState {
            phase: wrap_phase(
                here.phase + TIME_STEP_S * (here.natural_frequency + gain * coupling),
            ),
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
    let turn = TWO * FloatType::pi();
    let mut wrapped = phase;
    while wrapped >= turn {
        wrapped -= turn;
    }
    while wrapped < ZERO {
        wrapped += turn;
    }
    wrapped
}

/// Runs the connectome forward and returns its synchronisation.
pub fn simulate(brain: &Connectome) -> FloatType {
    let mut state = brain.clone();
    for _ in 0..SIMULATION_STEPS {
        state = kuramoto_step(&state);
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
        GraphWitness::fold(phasors, (ZERO, ZERO, 0u64), |acc, phasor| match phasor {
            Some((c, s)) => (acc.0 + c, acc.1 + s, acc.2 + 1),
            None => acc,
        });

    if counted == 0 {
        return ZERO;
    }
    Real::sqrt(sum_cos * sum_cos + sum_sin * sum_sin) / lift_count::<FloatType>(counted)
}

/// Whether a synchronisation reading counts as a seizure.
pub fn is_seizing(sync: FloatType) -> bool {
    sync > SEIZURE_THRESHOLD
}
