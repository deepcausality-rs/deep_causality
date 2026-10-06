/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the DDoS-detector corrective control loop.

#![allow(dead_code)] // Telemetry fields kept for narrative realism even if not all are read.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalityError, PropagatingProcess};
use deep_causality_data_structures::{ArrayStorage, SlidingWindow, window_type};

/// Networking telemetry is fine at `f64` throughout: there is no precision
/// argument for more or fewer bits when counting megabits per second.
pub type FloatType = f64;

/// Sliding-window size: the last `WINDOW_SIZE` admitted samples define
/// "normal". A wider window (30 s of one-second samples) yields a steadier
/// mean and σ — a single sample sways the baseline less the larger it is.
pub const WINDOW_SIZE: usize = 30;

/// Array-backed capacity. The window over-allocates ~2x its size so pushes
/// stay copy-free until a cheap rewind every `WINDOW_CAPACITY` writes.
pub const WINDOW_CAPACITY: usize = 60;

/// One tick is one second. Eighty ticks is long enough to fill the 30 s
/// baseline with clean traffic, then suffer a DoS surge, detect it, mitigate,
/// and show the throughput settle back under the throttle ceiling.
pub const N_TICKS: u32 = 80;

/// The virtual NIC's rate-limiter command, carried in the value channel. The
/// corrective intervention flips it from OFF to ON.
pub type ThrottleState = u8;
pub const THROTTLE_OFF: ThrottleState = 0;
pub const THROTTLE_ON: ThrottleState = 1;

/// The array-backed sliding window over per-tick throughput samples (Mbps).
pub type ThroughputWindow =
    SlidingWindow<ArrayStorage<FloatType, WINDOW_SIZE, WINDOW_CAPACITY>, FloatType>;

/// Construct an empty throughput window. The return type pins the const
/// generics so the call site needs no turbofish.
pub fn new_throughput_window() -> ThroughputWindow {
    window_type::new_with_array_storage()
}

/// One sample of interface telemetry, as an enterprise router exports it per
/// second. `throughput_mbps` is the analyzed signal; the rest is realistic
/// context that a real triage workflow would attach to the alert.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct InterfaceTelemetry {
    /// Ingress throughput in megabits per second — the analyzed signal.
    pub throughput_mbps: FloatType,
    pub packets_per_sec: FloatType,
    /// New connections per second (SYN rate); spikes under volumetric DoS.
    pub new_conns_per_sec: FloatType,
    pub active_flows: FloatType,
    pub avg_packet_bytes: FloatType,
    pub output_drop_pct: FloatType,
    pub control_cpu_pct: FloatType,
}

/// The payload of one detector-context node. Throughput levels, rates, packet sizes, ratios and
/// the sigma threshold are real magnitudes; slot counts, tick indices and tick durations are whole
/// ticks; flow counts are whole numbers. Each keeps its own type.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum DetectorFact {
    /// The default that `Data` requires of its payload. The builder sets every node to
    /// `Real`, `Ticks` or `Count`.
    #[default]
    Unset,
    Real(FloatType),
    Ticks(u32),
    Count(u32),
}

/// The baseline, the attack schedule, the traffic profile, the detection thresholds and the
/// mitigation ceiling the steps read, one `Data` contextoid per quantity. The context holds no position, clock or event,
/// so its spatial, temporal and spacetime slots are empty.
pub type DetectorContext =
    Context<Data<DetectorFact>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node index: mean offered throughput while nominal (Mbps).
pub const BASELINE_MBPS: usize = 0;
/// Node index: amplitude of the deterministic jitter on the baseline (Mbps).
pub const BASELINE_JITTER_MBPS: usize = 1;
/// Node index: anomaly threshold in standard deviations (z-score). 3.0 = 3 sigma.
pub const SIGMA_THRESHOLD: usize = 2;
/// Node index: consecutive anomalous slots (ticks) required before the loop intervenes.
pub const TRIGGER_SLOTS: usize = 3;
/// Node index: the tick at which the volumetric surge begins.
pub const ATTACK_START_TICK: usize = 4;
/// Node index: offered throughput at the height of the surge (Mbps).
pub const ATTACK_PEAK_MBPS: usize = 5;
/// Node index: throughput ceiling the NIC clamps to once throttling is engaged (Mbps).
pub const THROTTLE_CEILING_MBPS: usize = 6;
/// Node index: throughput above which a tick counts as a service overload (Mbps).
pub const OVERLOAD_LINE_MBPS: usize = 7;
/// Node index: overload ticks tolerated before the service objective is breached.
pub const OVERLOAD_BUDGET_TICKS: usize = 8;
/// Node index: angular frequency of the baseline jitter (radians per tick).
pub const JITTER_FREQUENCY: usize = 9;
/// Node index: ticks the surge takes to ramp from the baseline to the peak.
pub const RAMP_TICKS: usize = 10;
/// Node index: average packet size of nominal traffic (bytes).
pub const NOMINAL_PACKET_BYTES: usize = 11;
/// Node index: average packet size under the flood (bytes). A flood is dominated by small
/// packets.
pub const ATTACK_PACKET_BYTES: usize = 12;
/// Node index: new connections per second of nominal traffic.
pub const NOMINAL_NEW_CONNS_PER_SEC: usize = 13;
/// Node index: new connections per packet under the flood (a SYN-heavy mix).
pub const ATTACK_NEW_CONNS_PER_PACKET: usize = 14;
/// Node index: active flows of nominal traffic.
pub const NOMINAL_ACTIVE_FLOWS: usize = 15;
/// Node index: active flows under the flood.
pub const ATTACK_ACTIVE_FLOWS: usize = 16;
/// Node index: throughput at which the control-plane CPU saturates (Mbps).
pub const CPU_SATURATION_MBPS: usize = 17;

/// The nominal detector world, added in node-index order: node `i` holds contextoid id `i + 1`.
pub fn nominal_detector_context() -> Result<DetectorContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "detector", 18);
    for (id, fact) in [
        (1, DetectorFact::Real(400.0)),    // BASELINE_MBPS
        (2, DetectorFact::Real(15.0)),     // BASELINE_JITTER_MBPS
        (3, DetectorFact::Real(3.0)),      // SIGMA_THRESHOLD
        (4, DetectorFact::Ticks(5)),       // TRIGGER_SLOTS
        (5, DetectorFact::Ticks(40)),      // ATTACK_START_TICK
        (6, DetectorFact::Real(900.0)),    // ATTACK_PEAK_MBPS
        (7, DetectorFact::Real(420.0)),    // THROTTLE_CEILING_MBPS
        (8, DetectorFact::Real(480.0)),    // OVERLOAD_LINE_MBPS
        (9, DetectorFact::Ticks(8)),       // OVERLOAD_BUDGET_TICKS
        (10, DetectorFact::Real(0.7)),     // JITTER_FREQUENCY
        (11, DetectorFact::Ticks(4)),      // RAMP_TICKS
        (12, DetectorFact::Real(800.0)),   // NOMINAL_PACKET_BYTES
        (13, DetectorFact::Real(120.0)),   // ATTACK_PACKET_BYTES
        (14, DetectorFact::Real(200.0)),   // NOMINAL_NEW_CONNS_PER_SEC
        (15, DetectorFact::Real(0.5)),     // ATTACK_NEW_CONNS_PER_PACKET
        (16, DetectorFact::Count(1_200)),  // NOMINAL_ACTIVE_FLOWS
        (17, DetectorFact::Count(50_000)), // ATTACK_ACTIVE_FLOWS
        (18, DetectorFact::Real(1_500.0)), // CPU_SATURATION_MBPS
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, fact)),
        ))?;
    }
    Ok(context)
}

/// Read one `Data` contextoid's payload out of the detector context, or name the node it lacks.
fn read(context: &DetectorContext, index: usize) -> Result<DetectorFact, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| {
            CausalityError::MissingParameter(format!("detector context Datoid at node {index}"))
        })
}

/// Read a real magnitude (Mbps or sigma) out of the detector context.
pub fn read_real(context: &DetectorContext, index: usize) -> Result<FloatType, CausalityError> {
    match read(context, index)? {
        DetectorFact::Real(value) => Ok(value),
        other => Err(mismatch(index, "Real", other)),
    }
}

/// Read a whole number of ticks out of the detector context.
pub fn read_ticks(context: &DetectorContext, index: usize) -> Result<u32, CausalityError> {
    match read(context, index)? {
        DetectorFact::Ticks(ticks) => Ok(ticks),
        other => Err(mismatch(index, "Ticks", other)),
    }
}

/// Read a whole-number count out of the detector context.
pub fn read_count(context: &DetectorContext, index: usize) -> Result<u32, CausalityError> {
    match read(context, index)? {
        DetectorFact::Count(count) => Ok(count),
        other => Err(mismatch(index, "Count", other)),
    }
}

/// The error for a node whose fact has a different kind than its index promises.
fn mismatch(index: usize, expected: &str, found: DetectorFact) -> CausalityError {
    CausalityError::TypeConversionError(format!(
        "detector context node {index}: expected {expected}, found {found:?}"
    ))
}

/// Per-tick accounting plus the rolling-baseline sliding window. Holds the
/// `ThroughputWindow` directly: the monad's `bind` does not require
/// `State: Clone`, so the non-`Clone` window rides along as Markovian state.
///
/// No `#[derive(Debug)]`: `SlidingWindow` is not `Debug`, and the monad never
/// requires `State: Debug` (only `intervene` needs `Value: Debug`).
pub struct DetectorState {
    pub tick: u32,
    /// The rolling baseline of recently *admitted* (non-anomalous) throughput.
    pub window: ThroughputWindow,
    pub throughput_history: Vec<FloatType>,
    pub zscore_history: Vec<FloatType>,
    pub throttle_history: Vec<ThrottleState>,
    pub consecutive_anomalies: u32,
    pub mitigation_count: u32,
    /// Onset: the first anomalous second observed. The *confirmed* detection
    /// is `mitigated_at` (the trigger), which lands `trigger_slots` later.
    pub first_anomaly_at: Option<u32>,
    pub mitigated_at: Option<u32>,
    pub peak_throughput_mbps: FloatType,
    pub overload_ticks: u32,
    pub overload_threshold_reached_at: Option<u32>,
}

impl DetectorState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            window: new_throughput_window(),
            throughput_history: Vec::new(),
            zscore_history: Vec::new(),
            throttle_history: Vec::new(),
            consecutive_anomalies: 0,
            mitigation_count: 0,
            first_anomaly_at: None,
            mitigated_at: None,
            peak_throughput_mbps: 0.0,
            overload_ticks: 0,
            overload_threshold_reached_at: None,
        }
    }
}

impl Default for DetectorState {
    fn default() -> Self {
        Self::new()
    }
}

pub type DetectorProcess<T> = PropagatingProcess<T, DetectorState, DetectorContext>;
