/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Traffic generation, NIC rate-limiting, and the per-tick detector stage.

use crate::model_types::{
    DetectorConfig, DetectorProcess, DetectorState, FloatType, InterfaceTelemetry, THROTTLE_OFF,
    THROTTLE_ON, ThrottleState, ThroughputWindow, nominal_detector_config,
};
use causal_correction_examples::math_utils;
use deep_causality_core::{CausalEffect, EffectLog};
use deep_causality_haft::LogAddEntry;

/// Offered load presented to the interface at `tick`, before any
/// rate-limiting. Deterministic — a fixed sine jitter on the baseline plus a
/// ramped volumetric surge after `attack_start_tick` — so the whole run is
/// reproducible with no RNG, matching the other corrective examples.
pub fn offered_load(tick: u32, cfg: &DetectorConfig) -> InterfaceTelemetry {
    let jitter = cfg.baseline_jitter_mbps * (tick as FloatType * 0.7).sin();
    let mut throughput = cfg.baseline_mbps + jitter;

    let under_attack = tick >= cfg.attack_start_tick;
    if under_attack {
        // Ramp to peak over four seconds, then hold the flood.
        let ramp = ((tick - cfg.attack_start_tick) as FloatType / 4.0).min(1.0);
        throughput += (cfg.attack_peak_mbps - cfg.baseline_mbps) * ramp;
    }

    telemetry_from_throughput(throughput, under_attack)
}

/// Derive plausible per-interface counters from a throughput figure. A flood
/// is dominated by small packets and a high new-connection rate, so those
/// fields shift under attack. Only `throughput_mbps` feeds the detector.
fn telemetry_from_throughput(throughput_mbps: FloatType, under_attack: bool) -> InterfaceTelemetry {
    let avg_packet_bytes = if under_attack { 120.0 } else { 800.0 };
    let packets_per_sec = throughput_mbps * 1_000_000.0 / 8.0 / avg_packet_bytes;
    let new_conns_per_sec = if under_attack {
        packets_per_sec * 0.5
    } else {
        200.0
    };
    InterfaceTelemetry {
        throughput_mbps,
        packets_per_sec,
        new_conns_per_sec,
        active_flows: if under_attack { 50_000.0 } else { 1_200.0 },
        avg_packet_bytes,
        output_drop_pct: 0.0,
        control_cpu_pct: (throughput_mbps / 1_500.0 * 100.0).min(100.0),
    }
}

/// The virtual NIC's throughput regulator. When throttling is engaged it
/// clamps measured throughput to the configured ceiling — the mitigation
/// acting directly on the stream the window observes. A no-op while OFF or
/// when the offered load already sits under the ceiling.
pub fn regulate(
    sample: InterfaceTelemetry,
    throttle: ThrottleState,
    cfg: &DetectorConfig,
) -> InterfaceTelemetry {
    if throttle == THROTTLE_OFF || sample.throughput_mbps <= cfg.throttle_ceiling_mbps {
        return sample;
    }
    let scale = cfg.throttle_ceiling_mbps / sample.throughput_mbps;
    InterfaceTelemetry {
        throughput_mbps: cfg.throttle_ceiling_mbps,
        packets_per_sec: sample.packets_per_sec * scale,
        new_conns_per_sec: sample.new_conns_per_sec * scale,
        output_drop_pct: (1.0 - scale) * 100.0,
        ..sample
    }
}

/// Scale-invariant anomaly score for a *new* sample against the rolling
/// baseline window: how many standard deviations it sits above the window
/// mean. `None` until the window has filled.
///
/// Scoring the incoming sample against the baseline — rather than re-deriving
/// the mean and σ from a window that already contains the attack — is what
/// keeps a sustained surge detectable. A naive "max of the window exceeds
/// mean + 3σ of that same window" self-masks: as flood samples accumulate
/// they inflate the window's own mean and σ, and the z-score of the max
/// collapses back below the threshold within a few ticks. The detector
/// therefore admits only non-anomalous samples to the window (see
/// [`analyze_tick`]), so the baseline stays clean and the flood reads as
/// anomalous for as long as it lasts.
pub fn baseline_zscore(window: &ThroughputWindow, sample: FloatType) -> Option<FloatType> {
    if !window.filled() {
        return None;
    }
    let slice = window.slice().ok()?;
    let mean = math_utils::mean(slice);
    let n = slice.len() as FloatType;
    let variance = slice.iter().map(|&x| (x - mean).powi(2)).sum::<FloatType>() / (n - 1.0);
    let std = variance.sqrt();
    if std <= FloatType::EPSILON {
        // A perfectly flat baseline has no spread: anything off it is maximally
        // anomalous, anything on it is normal.
        return Some(if (sample - mean).abs() > FloatType::EPSILON {
            FloatType::INFINITY
        } else {
            0.0
        });
    }
    Some((sample - mean) / std)
}

/// One simulation tick. Reads the NIC throttle command from the value
/// channel, generates the offered load, rate-limits it, scores the measured
/// throughput against the rolling baseline, and admits the sample to the
/// baseline only if it is not anomalous. The throttle command is preserved in
/// the value channel so the monitor can intervene on it.
pub fn analyze_tick(
    value: CausalEffect<ThrottleState>,
    mut state: DetectorState,
    ctx: Option<DetectorConfig>,
) -> DetectorProcess<ThrottleState> {
    let cfg = ctx.as_ref().expect("DetectorConfig required");
    let throttle = value.into_value().unwrap_or(THROTTLE_OFF);

    let measured = regulate(offered_load(state.tick, cfg), throttle, cfg);
    let tp = measured.throughput_mbps;

    let z = baseline_zscore(&state.window, tp);
    let anomalous = z.is_some_and(|z| z > cfg.sigma_threshold);

    // Withhold anomalous samples so the flood never poisons the baseline.
    if !anomalous {
        state.window.push(tp);
    }

    state.tick += 1;
    state.throughput_history.push(tp);
    state.zscore_history.push(z.unwrap_or(0.0));
    state.throttle_history.push(throttle);
    if tp > state.peak_throughput_mbps {
        state.peak_throughput_mbps = tp;
    }

    if anomalous {
        state.consecutive_anomalies += 1;
        // First anomalous second of the surge — the onset, distinct from the
        // confirmed detection, which is the trigger (`mitigated_at` in `main`,
        // set once `consecutive_anomalies` reaches `trigger_slots`).
        if state.first_anomaly_at.is_none() {
            state.first_anomaly_at = Some(state.tick);
        }
    } else {
        state.consecutive_anomalies = 0;
    }

    if tp > cfg.overload_line_mbps {
        state.overload_ticks += 1;
        if state.overload_threshold_reached_at.is_none()
            && state.overload_ticks >= cfg.overload_budget_ticks
        {
            state.overload_threshold_reached_at = Some(state.tick);
        }
    }

    let mut logs = EffectLog::new();
    let marker = if throttle == THROTTLE_ON {
        " [THROTTLED]"
    } else if anomalous {
        " [ANOMALY]"
    } else {
        ""
    };
    logs.add_entry(&format!(
        "tick {:>2}: throughput = {:>6.0} Mbps, z = {:>6.2}, throttle = {}{}",
        state.tick,
        tp,
        z.unwrap_or(0.0),
        if throttle == THROTTLE_ON { "ON" } else { "OFF" },
        marker
    ));

    DetectorProcess::<ThrottleState>::new(Ok(CausalEffect::value(throttle)), state, ctx, logs)
}

pub fn initial_process() -> DetectorProcess<ThrottleState> {
    DetectorProcess::<ThrottleState>::new(
        Ok(CausalEffect::value(THROTTLE_OFF)),
        DetectorState::new(),
        Some(nominal_detector_config()),
        EffectLog::new(),
    )
}
