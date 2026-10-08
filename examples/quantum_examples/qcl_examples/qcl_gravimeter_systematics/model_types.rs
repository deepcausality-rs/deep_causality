/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The types the gravimeter-systematics example is built from.

use crate::FloatType;
use deep_causality_context::Context;
use deep_causality_context_store::ContextSnapshot;
use deep_causality_quantum::{InterferometerContext, ObservedContext, QuantumError};

/// A systematic effect that shifts the gravimeter's reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Systematic {
    /// Coriolis acceleration from the atoms' mean transverse velocity.
    Coriolis,
    /// The quadratic Zeeman shift from the field's inhomogeneity along the trajectory.
    QuadraticZeeman,
    /// A tilt of the beam axis from vertical.
    Tilt,
    /// Vibration of the reference mirror synchronous with the cycle.
    MirrorVibration,
    /// Aberration of the Raman beams' wavefront.
    Wavefront,
    /// The two-photon light shift from off-resonant Raman transitions.
    LightShift,
    /// Detection clipping combined with an offset of the cloud's starting position.
    Clipping,
}

/// A cause: a systematic and its size. The size is the offset the systematic produces in the
/// passive configuration, in m/s², except for a tilt, whose size is its angle from vertical in
/// rad.
#[derive(Debug, Clone, PartialEq)]
pub struct Cause {
    /// The name the verdict reports.
    pub name: String,
    /// The systematic.
    pub systematic: Systematic,
    /// Its size.
    pub size: FloatType,
}

/// Which tide a world's prediction carries: the reading the context records, as a candidate's
/// does, or the tide at the experiment's time, as the world the observations come from does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tide {
    /// The Earth-tide reading the context records.
    Recorded,
    /// The Earth tide at the experiment's time.
    Actual,
}

/// The numbers every response is built from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Physics {
    /// The fringe contrast.
    pub contrast: FloatType,
    /// Standard gravity, in m/s².
    pub gravity: FloatType,
    /// The operating bias field, in T.
    pub bias_field: FloatType,
    /// The operating Rabi frequency, in rad/s.
    pub rabi_frequency: FloatType,
    /// The quadratic Zeeman bias's shares `(a, b, c)` at nominal current.
    pub zeeman_shares: (FloatType, FloatType, FloatType),
    /// The share of the two-photon light shift a field step moves.
    pub light_shift_field_share: FloatType,
    /// The clipping shift per m of initial cloud displacement, in s⁻².
    pub clipping_slope: FloatType,
    /// The tide's amplitude, in m/s², its phase at the start, in rad, and its period, in s.
    pub tide: (FloatType, FloatType, FloatType),
}

/// The instrument context an experiment runs in, and the tick, in s from the start, it runs at.
#[derive(Debug, Clone)]
pub struct Session {
    /// The context: the instrument model, the configuration, and the reading taken at `tick`.
    pub context: InterferometerContext<FloatType>,
    /// When the experiment runs.
    pub tick: u64,
}

/// An observation records the session's context.
impl ObservedContext for Session {
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError> {
        <Context<_, _, _, _> as ObservedContext>::context_snapshot(&self.context)
    }
}

/// How a run ended.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// One cause holds and separates from every rival.
    Survivor(String),
    /// No cause holds.
    OutsideTheModel,
    /// Causes remain that nothing offered separates.
    Unresolved,
}

/// One run: the world the observations come from and what the lab knows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scenario {
    /// What the run shows.
    pub label: &'static str,
    /// The systematic the observations come from.
    pub truth: Systematic,
    /// The offset it produces in the passive configuration, in µGal.
    pub offset_ugal: FloatType,
    /// Whether the contexts record the Earth tide.
    pub records_tide: bool,
    /// A systematic left off the candidate list, if one is.
    pub without: Option<Systematic>,
}

/// What one run concluded and spent.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldRun {
    /// The scenario run.
    pub scenario: Scenario,
    /// The candidates the baseline refused, each with its predicted and the observed read-out.
    pub refused: Vec<(String, FloatType, FloatType)>,
    /// The static plan's experiments, in order.
    pub plan: Vec<String>,
    /// What the static plan costs, in s.
    pub plan_cost: FloatType,
    /// Whether the plan separates every pair at the floor.
    pub plan_complete: bool,
    /// The candidates no planned experiment moves, which the plan identifies by elimination.
    pub eliminated: Vec<String>,
    /// The experiments the campaign ran, in order.
    pub campaign: Vec<String>,
    /// What the campaign spent, in s.
    pub campaign_cost: FloatType,
    /// How often the tide forced the campaign to re-plan.
    pub replans: usize,
    /// The campaign's verdict.
    pub campaign_verdict: Verdict,
}
