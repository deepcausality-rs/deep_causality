/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Traffic generation, NIC rate-limiting, and the per-tick detector stage.

use crate::model_types::{
    ATTACK_ACTIVE_FLOWS, ATTACK_NEW_CONNS_PER_PACKET, ATTACK_PACKET_BYTES, ATTACK_PEAK_MBPS,
    ATTACK_START_TICK, BASELINE_JITTER_MBPS, BASELINE_MBPS, CPU_SATURATION_MBPS, DetectorContext,
    DetectorProcess, DetectorState, FloatType, InterfaceTelemetry, JITTER_FREQUENCY,
    NOMINAL_ACTIVE_FLOWS, NOMINAL_NEW_CONNS_PER_SEC, NOMINAL_PACKET_BYTES, OVERLOAD_BUDGET_TICKS,
    OVERLOAD_LINE_MBPS, RAMP_TICKS, SIGMA_THRESHOLD, THROTTLE_CEILING_MBPS, THROTTLE_OFF,
    THROTTLE_ON, TRIGGER_SLOTS, ThrottleState, ThroughputWindow, read_count, read_real, read_ticks,
};
use deep_causality_core::{CausalEffect, CausalityError, EffectLog};
use deep_causality_haft::LogAddEntry;
use deep_causality_num::Lift;

/// Offered load presented to the interface at `tick`, before any
/// rate-limiting. Deterministic — a fixed sine jitter on the baseline plus a
/// ramped volumetric surge after `attack_start_tick` — so the whole run is
/// reproducible with no RNG, matching the other corrective examples. The
/// baseline, the jitter, the attack schedule and the ramp are read from
/// `detector`.
pub fn offered_load(
    tick: u32,
    detector: &DetectorContext,
) -> Result<InterfaceTelemetry, CausalityError> {
    let baseline_mbps = read_real(detector, BASELINE_MBPS)?;
    let attack_start_tick = read_ticks(detector, ATTACK_START_TICK)?;
    let jitter = read_real(detector, BASELINE_JITTER_MBPS)?
        * (tick.lift::<FloatType>() * read_real(detector, JITTER_FREQUENCY)?).sin();
    let mut throughput = baseline_mbps + jitter;

    let under_attack = tick >= attack_start_tick;
    if under_attack {
        // Ramp to peak over `RAMP_TICKS` seconds, then hold the flood.
        let ramp_ticks = read_ticks(detector, RAMP_TICKS)?.lift::<FloatType>();
        let ramp = ((tick - attack_start_tick).lift::<FloatType>() / ramp_ticks).min(1.0);
        throughput += (read_real(detector, ATTACK_PEAK_MBPS)? - baseline_mbps) * ramp;
    }

    telemetry_from_throughput(throughput, under_attack, detector)
}

/// Derive plausible per-interface counters from a throughput figure. A flood
/// is dominated by small packets and a high new-connection rate, so those
/// fields shift under attack. Only `throughput_mbps` feeds the detector. The
/// packet sizes, connection rates, flow counts and the CPU saturation point
/// are read from `detector`.
fn telemetry_from_throughput(
    throughput_mbps: FloatType,
    under_attack: bool,
    detector: &DetectorContext,
) -> Result<InterfaceTelemetry, CausalityError> {
    let avg_packet_bytes = if under_attack {
        read_real(detector, ATTACK_PACKET_BYTES)?
    } else {
        read_real(detector, NOMINAL_PACKET_BYTES)?
    };
    // Mbps to bytes per second: 10^6 bits per megabit, 8 bits per byte.
    let packets_per_sec = throughput_mbps * 1_000_000.0 / 8.0 / avg_packet_bytes;
    let new_conns_per_sec = if under_attack {
        packets_per_sec * read_real(detector, ATTACK_NEW_CONNS_PER_PACKET)?
    } else {
        read_real(detector, NOMINAL_NEW_CONNS_PER_SEC)?
    };
    let active_flows = if under_attack {
        read_count(detector, ATTACK_ACTIVE_FLOWS)?
    } else {
        read_count(detector, NOMINAL_ACTIVE_FLOWS)?
    };
    Ok(InterfaceTelemetry {
        throughput_mbps,
        packets_per_sec,
        new_conns_per_sec,
        active_flows: active_flows.lift::<FloatType>(),
        avg_packet_bytes,
        output_drop_pct: 0.0,
        // Throughput as a percentage of the saturation point, capped at 100 %.
        control_cpu_pct: (throughput_mbps / read_real(detector, CPU_SATURATION_MBPS)? * 100.0)
            .min(100.0),
    })
}

/// The virtual NIC's throughput regulator. When throttling is engaged it
/// clamps measured throughput to the configured ceiling — the mitigation
/// acting directly on the stream the window observes. A no-op while OFF or
/// when the offered load already sits under the ceiling.
pub fn regulate(
    sample: InterfaceTelemetry,
    throttle: ThrottleState,
    detector: &DetectorContext,
) -> Result<InterfaceTelemetry, CausalityError> {
    let throttle_ceiling_mbps = read_real(detector, THROTTLE_CEILING_MBPS)?;
    if throttle == THROTTLE_OFF || sample.throughput_mbps <= throttle_ceiling_mbps {
        return Ok(sample);
    }
    let scale = throttle_ceiling_mbps / sample.throughput_mbps;
    Ok(InterfaceTelemetry {
        throughput_mbps: throttle_ceiling_mbps,
        packets_per_sec: sample.packets_per_sec * scale,
        new_conns_per_sec: sample.new_conns_per_sec * scale,
        output_drop_pct: (1.0 - scale) * 100.0,
        ..sample
    })
}

/// Scale-invariant anomaly score for a *new* sample against the rolling
/// baseline window: how many standard deviations it sits above the window
/// mean. `None` until the window has filled; an error if the filled window
/// cannot be read or its statistics refuse.
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
pub fn baseline_zscore(
    window: &ThroughputWindow,
    sample: FloatType,
) -> Result<Option<FloatType>, CausalityError> {
    if !window.filled() {
        return Ok(None);
    }
    let slice = window.slice().map_err(CausalityError::ModelError)?;
    // The window is `filled()`, so it carries `WINDOW_SIZE` samples and neither statistic can
    // refuse for want of observations; the corrected `n − 1` form is what this z-score wants.
    let mean = deep_causality_stats::mean(slice)
        .map_err(|err| CausalityError::ModelError(err.to_string()))?;
    let std = deep_causality_stats::std_dev(slice)
        .map_err(|err| CausalityError::ModelError(err.to_string()))?;
    if std <= FloatType::EPSILON {
        // A perfectly flat baseline has no spread: anything off it is maximally
        // anomalous, anything on it is normal.
        return Ok(Some(if (sample - mean).abs() > FloatType::EPSILON {
            FloatType::INFINITY
        } else {
            0.0
        }));
    }
    Ok(Some((sample - mean) / std))
}

/// One simulation tick. Reads the NIC throttle command from the value
/// channel, generates the offered load, rate-limits it, scores the measured
/// throughput against the rolling baseline, and admits the sample to the
/// baseline only if it is not anomalous. The throttle command is preserved in
/// the value channel so the monitor can intervene on it. The baseline, the
/// attack schedule, the traffic profile, the thresholds and the trigger slot
/// count are read from the detector context the process carries. A missing
/// throttle command, context or detector fact, or a baseline that cannot be
/// scored, ends the process in error with the state unchanged.
pub fn analyze_tick(
    value: CausalEffect<ThrottleState>,
    mut state: DetectorState,
    ctx: Option<DetectorContext>,
) -> DetectorProcess<ThrottleState> {
    match advance(value, &mut state, ctx.as_ref()) {
        Ok((throttle, logs)) => DetectorProcess::<ThrottleState>::new(
            Ok(CausalEffect::value(throttle)),
            state,
            ctx,
            logs,
        ),
        Err(err) => DetectorProcess::<ThrottleState>::new(Err(err), state, ctx, EffectLog::new()),
    }
}

/// The tick itself: reads and scores every input first, then advances
/// `state` and returns the throttle command with its log entry.
fn advance(
    value: CausalEffect<ThrottleState>,
    state: &mut DetectorState,
    ctx: Option<&DetectorContext>,
) -> Result<(ThrottleState, EffectLog), CausalityError> {
    let throttle = value
        .into_value()
        .ok_or_else(CausalityError::ValueNotAvailable)?;
    let detector = ctx.ok_or_else(CausalityError::MissingContext)?;

    let measured = regulate(offered_load(state.tick, detector)?, throttle, detector)?;
    let tp = measured.throughput_mbps;

    let z = baseline_zscore(&state.window, tp)?;
    let sigma_threshold = read_real(detector, SIGMA_THRESHOLD)?;
    let overload_line_mbps = read_real(detector, OVERLOAD_LINE_MBPS)?;
    let overload_budget_ticks = read_ticks(detector, OVERLOAD_BUDGET_TICKS)?;
    // The trigger predicate in `main` reads the trigger slot count and has no error channel. This
    // read fails the tick on a context that lacks it, before the predicate runs.
    read_ticks(detector, TRIGGER_SLOTS)?;
    let anomalous = z.is_some_and(|z| z > sigma_threshold);

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

    if tp > overload_line_mbps {
        state.overload_ticks += 1;
        if state.overload_threshold_reached_at.is_none()
            && state.overload_ticks >= overload_budget_ticks
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

    Ok((throttle, logs))
}

/// Initial process with the throttle off, carrying `detector` as its context.
pub fn initial_process(detector: DetectorContext) -> DetectorProcess<ThrottleState> {
    DetectorProcess::<ThrottleState>::new(
        Ok(CausalEffect::value(THROTTLE_OFF)),
        DetectorState::new(),
        Some(detector),
        EffectLog::new(),
    )
}
