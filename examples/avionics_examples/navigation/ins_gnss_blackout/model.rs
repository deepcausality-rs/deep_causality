/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model for the INS / GNSS-blackout clock-holdover example (a general GPS-denial scenario).
//!
//! Composes four native mechanisms over real Galileo data:
//! * **`deep_causality_file`** loads the real E14 SP3 orbit + `.clk` clock (the GNSS signal).
//! * the **`relativistic_clock_drift_rate_kernel`** predicts the clock rate from the real orbit
//!   geometry — the model that is *carried* across the outage.
//! * the **`select_metric` pattern** (from the grmhd example) detects the GNSS-denial regime from a
//!   denial indicator (interference / jamming / shadowing level) vs a critical threshold.
//! * the **`intervene` / `branch_with`** corrective loop applies the GNSS fix when available and
//!   *withholds* it during the blackout — the chain runs open-loop (drift) through the dark.
use crate::FloatType;
use chrono::NaiveDateTime;
use deep_causality_core::{EffectLog, EffectValue, PropagatingProcess};
use deep_causality_file::{ClockData, OrbitData};
use deep_causality_haft::LogAddEntry;
use deep_causality_physics::{EARTH_GM, relativistic_clock_drift_rate_kernel};

pub type NavProcess = PropagatingProcess<FloatType, NavState, NavConfig>;

/// One processed epoch of the real GNSS stream.
#[derive(Debug, Clone)]
pub struct Epoch {
    /// Seconds since the first epoch.
    pub t_sec: FloatType,
    /// Δt to the next epoch (s).
    pub dt: FloatType,
    /// Orbital radius from Earth centre (m), from the real SP3 position.
    pub radius_m: FloatType,
    /// Speed (m/s), finite-differenced from the real SP3 positions.
    pub speed_ms: FloatType,
    /// Measured satellite clock offset (ns), from the real `.clk` product — the ground truth.
    pub measured_clock_ns: FloatType,
    /// Relativistic clock rate `dτ/dt − 1` (dimensionless), from the shipped kernel on the real orbit.
    pub relativistic_rate: FloatType,
    /// Synthetic GNSS-denial indicator (interference / jamming / signal-shadowing level) — the
    /// regime-detector input.
    pub denial_indicator: FloatType,
}

/// Read-only configuration carried in the `Context` channel.
#[derive(Debug, Clone)]
pub struct NavConfig {
    /// The processed real-GNSS epoch stream.
    pub stream: Vec<Epoch>,
    /// Critical denial level above which GNSS is treated as denied (the grmhd-style threshold).
    pub blackout_threshold: FloatType,
    /// GNSS position-fix gain (P-controller on the INS error; corrected = err·(1 − gain)).
    pub gps_gain: FloatType,
    /// Fraction of the residual accelerometer bias each fix calibrates away.
    pub bias_cal_gain: FloatType,
}

/// Mutable per-tick state carried in the `State` channel.
#[derive(Debug, Default, Clone)]
pub struct NavState {
    pub idx: usize,
    pub ins_vel_err: FloatType,
    pub ins_residual_bias: FloatType,
    pub carried_clock_ns: FloatType,
    pub naive_clock_ns: FloatType,
    pub last_rate_ns_per_s: FloatType,
    pub gnss_denied: bool,
    pub prev_denied: bool,
    pub regime_changes: u32,
    pub correction_count: u32,
    pub pre_blackout_err: FloatType,
    pub peak_blackout_err: FloatType,
    pub post_reacq_err: FloatType,
    pub clock_carry_err_relativistic: FloatType,
    pub clock_carry_err_naive: FloatType,
    pub final_err: FloatType,
}

// ── Real-data stream preparation ─────────────────────────────────────────────────────────────────

fn time_diff_secs(a: NaiveDateTime, b: NaiveDateTime) -> f64 {
    (a - b).num_milliseconds() as f64 / 1000.0
}

/// Build the processed epoch stream from the real SP3 orbit + `.clk` clock series: radius and speed
/// from the orbit, the nearest measured clock sample, the relativistic rate from the kernel, and a
/// synthetic denial-indicator bump over the chosen outage window `[outage_lo, outage_hi]` (fractions),
/// standing in for a GNSS-denied stretch — jamming, an urban canyon, a tunnel, or terrain shadowing.
pub fn build_stream(
    mut orbits: Vec<OrbitData<FloatType>>,
    clocks: Vec<ClockData<FloatType>>,
    outage_lo: f64,
    outage_hi: f64,
) -> Vec<Epoch> {
    orbits.sort_by_key(|o| o.timestamp());
    let n = orbits.len();
    if n < 3 {
        return Vec::new();
    }
    let t0 = orbits[0].timestamp();
    let centre = 0.5 * (outage_lo + outage_hi);
    let half_width = (0.5 * (outage_hi - outage_lo)).max(1e-6);
    let billion = 1.0e9;

    let mut stream = Vec::with_capacity(n);
    for i in 0..n {
        let o = &orbits[i];
        let radius_m = o.radius_m();

        // Central finite-difference speed (forward/backward at the ends).
        let a = orbits[i.saturating_sub(1)].clone();
        let b = orbits[(i + 1).min(n - 1)].clone();
        let dx = b.x_m() - a.x_m();
        let dy = b.y_m() - a.y_m();
        let dz = b.z_m() - a.z_m();
        let dt_fd = time_diff_secs(b.timestamp(), a.timestamp()).max(1.0);
        let speed_ms = (dx * dx + dy * dy + dz * dz).sqrt() / dt_fd;

        let dt = if i + 1 < n {
            time_diff_secs(orbits[i + 1].timestamp(), o.timestamp()).max(1.0)
        } else {
            time_diff_secs(o.timestamp(), orbits[i - 1].timestamp()).max(1.0)
        };
        let t_sec = time_diff_secs(o.timestamp(), t0);

        // Nearest measured clock sample (real .clk), in ns.
        let measured_clock_ns = clocks
            .iter()
            .min_by(|c1, c2| {
                let d1 = (c1.timestamp() - o.timestamp()).num_milliseconds().abs();
                let d2 = (c2.timestamp() - o.timestamp()).num_milliseconds().abs();
                d1.cmp(&d2)
            })
            .map(|c| c.bias_s() * billion)
            .unwrap_or(0.0);

        let relativistic_rate =
            relativistic_clock_drift_rate_kernel(radius_m, speed_ms, EARTH_GM).unwrap_or(0.0);

        // Synthetic denial indicator: a Gaussian bump centred on the outage window — the signal
        // degradation rising as the vehicle enters a GNSS-denied stretch and falling as it leaves.
        let frac = i as f64 / (n - 1) as f64;
        let z = (frac - centre) / half_width;
        let denial_indicator = (-(z * z)).exp();

        stream.push(Epoch {
            t_sec,
            dt,
            radius_m,
            speed_ms,
            measured_clock_ns,
            relativistic_rate,
            denial_indicator,
        });
    }
    stream
}

// ── CausalFlow stages ────────────────────────────────────────────────────────────────────────────

/// One tick: advance the INS error and carry both clocks one epoch. The value channel carries the INS
/// position error; the clocks and bookkeeping ride in `State`. Pure (no IO).
pub fn advance(
    value: EffectValue<FloatType>,
    mut state: NavState,
    ctx: Option<NavConfig>,
) -> NavProcess {
    let cfg = ctx.clone().expect("NavConfig required");
    let idx = state.idx.min(cfg.stream.len().saturating_sub(1));
    let ep = cfg.stream[idx].clone();
    let prev_err = value.into_value().unwrap_or(0.0);

    // INS error dynamics: a residual accelerometer bias integrates into velocity then position error.
    state.ins_vel_err += state.ins_residual_bias * ep.dt;
    let pos_err = prev_err + state.ins_vel_err * ep.dt;

    // Both clocks always advance by their model rate; the GNSS-available arm disciplines them back.
    let rel_rate_ns = ep.relativistic_rate * 1.0e9; // ns per second
    state.carried_clock_ns += rel_rate_ns * ep.dt;
    state.naive_clock_ns += state.last_rate_ns_per_s * ep.dt;

    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "epoch {:>3} t={:>6.0}s r={:.0}km v={:.0}m/s denial~{:.2}",
        idx,
        ep.t_sec,
        ep.radius_m / 1000.0,
        ep.speed_ms,
        ep.denial_indicator
    ));

    state.idx += 1;
    NavProcess::new(Ok(EffectValue::Value(pos_err)), state, ctx, logs)
}

/// The grmhd `select_metric` pattern: compute the regime from a state indicator (the GNSS-denial
/// level) vs a config threshold, set `state.gnss_denied`, and log the two regime changes (entry / exit).
pub fn detect_regime(
    value: EffectValue<FloatType>,
    mut state: NavState,
    ctx: Option<NavConfig>,
) -> NavProcess {
    let cfg = ctx.clone().expect("NavConfig required");
    let idx = state
        .idx
        .saturating_sub(1)
        .min(cfg.stream.len().saturating_sub(1));
    let denial = cfg.stream[idx].denial_indicator;
    let denied = denial > cfg.blackout_threshold;

    let mut logs = EffectLog::new();
    if denied != state.prev_denied {
        state.regime_changes += 1;
        let (i, thr) = (denial, cfg.blackout_threshold);
        if denied {
            logs.add_entry(&format!(
                "!! REGIME CHANGE: GNSS DENIED (denial {i:.2} > {thr:.2}) — INS dead-reckoning, carrying relativistic clock"
            ));
        } else {
            logs.add_entry(&format!(
                "!! REGIME CHANGE: GNSS RESTORED (denial {i:.2} <= {thr:.2}) — reacquire + re-discipline clock"
            ));
        }
    }
    state.gnss_denied = denied;
    state.prev_denied = denied;

    NavProcess::new(Ok(value), state, ctx, logs)
}

/// The corrective GNSS fix (closed loop, GNSS available): snap the INS position error toward zero with
/// the P-gain — this is the value the `intervene` call substitutes.
pub fn gps_fix(err: FloatType, cfg: &NavConfig) -> FloatType {
    err * (1.0 - cfg.gps_gain)
}

/// State-side effect of a GNSS fix: calibrate the accelerometer bias and velocity error down, discipline
/// both carried clocks to the measured value, freeze the naive last-rate, and count the intervention.
pub fn apply_fix(mut state: NavState, cfg: &NavConfig) -> NavState {
    let idx = state
        .idx
        .saturating_sub(1)
        .min(cfg.stream.len().saturating_sub(1));
    let ep = &cfg.stream[idx];
    state.ins_residual_bias *= 1.0 - cfg.bias_cal_gain;
    state.ins_vel_err *= 1.0 - cfg.gps_gain;
    // Discipline both clocks to the real measured clock; latch the current relativistic rate for the
    // naive hold that takes over if GNSS is lost next tick.
    state.carried_clock_ns = ep.measured_clock_ns;
    state.naive_clock_ns = ep.measured_clock_ns;
    state.last_rate_ns_per_s = ep.relativistic_rate * 1.0e9;
    state.correction_count += 1;
    state
}

/// Harvest per-tick metrics (called each tick from the output stage).
pub fn record_metrics(mut state: NavState, value: &FloatType, cfg: &NavConfig) -> NavState {
    let err = value.abs();
    let idx = state
        .idx
        .saturating_sub(1)
        .min(cfg.stream.len().saturating_sub(1));
    if state.gnss_denied {
        if err > state.peak_blackout_err {
            state.peak_blackout_err = err;
        }
        let cm = cfg.stream[idx].measured_clock_ns;
        state.clock_carry_err_relativistic = (state.carried_clock_ns - cm).abs();
        state.clock_carry_err_naive = (state.naive_clock_ns - cm).abs();
    } else if state.regime_changes == 0 {
        state.pre_blackout_err = err;
    } else if state.regime_changes >= 2 {
        state.post_reacq_err = err;
    }
    state.final_err = err;
    state
}

/// The initial process at the first epoch: zero INS error, the true (uncalibrated) accelerometer bias
/// loaded as the residual, clocks disciplined to the first measured sample.
pub fn initial_process(cfg: NavConfig, true_bias: FloatType) -> NavProcess {
    let first_clock = cfg
        .stream
        .first()
        .map(|e| e.measured_clock_ns)
        .unwrap_or(0.0);
    let first_rate = cfg
        .stream
        .first()
        .map(|e| e.relativistic_rate * 1.0e9)
        .unwrap_or(0.0);
    NavProcess::new(
        Ok(EffectValue::Value(0.0)),
        NavState {
            ins_residual_bias: true_bias,
            carried_clock_ns: first_clock,
            naive_clock_ns: first_clock,
            last_rate_ns_per_s: first_rate,
            ..NavState::default()
        },
        Some(cfg),
        EffectLog::new(),
    )
}
