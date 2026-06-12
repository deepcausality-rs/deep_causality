/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the DDoS-detector corrective control loop.

#![allow(dead_code)] // Telemetry fields kept for narrative realism even if not all are read.

use deep_causality_core::PropagatingProcess;
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

/// Read-only detector configuration: the baseline, the failure schedule, the
/// detection thresholds, and the mitigation ceiling.
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    pub baseline_mbps: FloatType,
    pub baseline_jitter_mbps: FloatType,
    /// Anomaly threshold in standard deviations (z-score). 3.0 = 3 sigma.
    pub sigma_threshold: FloatType,
    /// Consecutive anomalous slots required before the loop intervenes.
    pub trigger_slots: u32,
    /// The tick at which the volumetric surge begins.
    pub attack_start_tick: u32,
    pub attack_peak_mbps: FloatType,
    /// Throughput ceiling the NIC clamps to once throttling is engaged.
    pub throttle_ceiling_mbps: FloatType,
    /// Throughput above which a tick counts as a service overload.
    pub overload_line_mbps: FloatType,
    /// Overload ticks tolerated before the service objective is breached.
    pub overload_budget_ticks: u32,
}

pub fn nominal_detector_config() -> DetectorConfig {
    DetectorConfig {
        baseline_mbps: 400.0,
        baseline_jitter_mbps: 15.0,
        sigma_threshold: 3.0,
        trigger_slots: 5,
        attack_start_tick: 40,
        attack_peak_mbps: 900.0,
        throttle_ceiling_mbps: 420.0,
        overload_line_mbps: 480.0,
        overload_budget_ticks: 8,
    }
}

/// Per-tick accounting plus the rolling-baseline sliding window. Holds the
/// `ThroughputWindow` directly: now that the monad's `bind` no longer demands
/// `State: Clone`, a non-`Clone` window can ride along as Markovian state.
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
    pub attack_detected_at: Option<u32>,
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
            attack_detected_at: None,
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

pub type DetectorProcess<T> = PropagatingProcess<T, DetectorState, DetectorConfig>;
