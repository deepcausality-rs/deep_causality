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
//! * the **`alternate_value` / `branch_with`** corrective loop applies the GNSS fix when available
//!   and *withholds* it during the blackout — the chain runs open-loop (drift) through the dark.
//!
//! The navigation world (the processed epoch series, the outage window, the accelerometer bias,
//! the denial threshold, and the two fix gains) is a [`NavContext`] carried in the `Context`
//! channel; every stage reads it from there.
use crate::FloatType;
use chrono::NaiveDateTime;
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{
    AlternatableValue, CausalEffect, CausalityError, EffectLog, PropagatingProcess,
};
use deep_causality_file::{ClockData, OrbitData};
use deep_causality_haft::LogAddEntry;
use deep_causality_physics::{EARTH_GM, PhysicsError, relativistic_clock_drift_rate_kernel};
use std::rc::Rc;

pub type NavProcess = PropagatingProcess<FloatType, NavState, NavContext>;

/// One quantity of the navigation world: a per-epoch series of the real GNSS stream, or a scalar.
#[derive(Debug, Clone, PartialEq)]
pub enum NavDatum {
    /// One value per epoch, in epoch order. Shared, so reading a series clones a pointer.
    Series(Rc<[FloatType]>),
    /// A single value.
    Scalar(FloatType),
}

impl Default for NavDatum {
    fn default() -> Self {
        NavDatum::Scalar(FloatType::default())
    }
}

/// The navigation world, one `Data` contextoid per quantity. The epoch time stamps are a series
/// of GPS-time seconds and the orbit enters only as radius and speed, so the spatial, temporal and
/// spacetime slots are empty.
pub type NavContext = Context<Data<NavDatum>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Number of per-epoch series [`build_stream`] produces.
pub const EPOCH_SERIES: usize = 6;
/// Contextoid id (series): seconds since the first epoch, s.
pub const T_SEC: ContextoidId = 1;
/// Contextoid id (series): Δt to the next epoch, s.
pub const DT: ContextoidId = 2;
/// Contextoid id (series): orbital radius from Earth centre, m, from the real SP3 position.
pub const RADIUS_M: ContextoidId = 3;
/// Contextoid id (series): speed, m/s, finite-differenced from the real SP3 positions.
pub const SPEED_MS: ContextoidId = 4;
/// Contextoid id (series): measured satellite clock offset, ns, from the real `.clk` product — the
/// ground truth.
pub const MEASURED_CLOCK_NS: ContextoidId = 5;
/// Contextoid id (series): relativistic clock rate `dτ/dt − 1` (dimensionless), from the shipped
/// kernel on the real orbit.
pub const RELATIVISTIC_RATE: ContextoidId = 6;

/// Contextoid id (scalar): start of the GNSS outage window, as a fraction of the day's epochs.
pub const OUTAGE_START: ContextoidId = 7;
/// Contextoid id (scalar): end of the GNSS outage window, as a fraction of the day's epochs.
pub const OUTAGE_END: ContextoidId = 8;
/// Contextoid id (scalar): true accelerometer bias, m/s² (~10 µg, navigation grade).
pub const ACCEL_BIAS: ContextoidId = 9;
/// Contextoid id (scalar): critical denial level above which GNSS is treated as denied (the
/// grmhd-style threshold).
pub const BLACKOUT_THRESHOLD: ContextoidId = 10;
/// Contextoid id (scalar): GNSS position-fix gain (P-controller on the INS error;
/// corrected = err·(1 − gain)).
pub const GPS_GAIN: ContextoidId = 11;
/// Contextoid id (scalar): fraction of the residual accelerometer bias each fix calibrates away.
pub const BIAS_CAL_GAIN: ContextoidId = 12;

/// Build the navigation world from the processed epoch series and the scenario scalars, each keyed
/// by its contextoid id.
pub fn nav_world(
    series: [(ContextoidId, Vec<FloatType>); EPOCH_SERIES],
    scenario: &[(ContextoidId, FloatType)],
) -> Result<NavContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "navigation world", series.len() + scenario.len());
    let facts = series
        .into_iter()
        .map(|(id, values)| (id, NavDatum::Series(values.into())))
        .chain(
            scenario
                .iter()
                .map(|&(id, value)| (id, NavDatum::Scalar(value))),
        );
    for (id, datum) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, datum)),
        ))?;
    }
    Ok(context)
}

/// The navigation world in the `Context` channel, or the error a stage returns when it is empty.
fn world(ctx: Option<&NavContext>) -> Result<&NavContext, CausalityError> {
    ctx.ok_or_else(CausalityError::MissingContext)
}

/// Read the `Data` contextoid carrying `id` out of the navigation world.
fn datum(context: &NavContext, id: ContextoidId) -> Result<NavDatum, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError::MissingParameter(format!(
            "navigation world holds no Datoid with contextoid id {id}"
        ))
    })
}

/// Read one per-epoch series out of the navigation world.
pub fn series(context: &NavContext, id: ContextoidId) -> Result<Rc<[FloatType]>, CausalityError> {
    match datum(context, id)? {
        NavDatum::Series(values) => Ok(values),
        NavDatum::Scalar(_) => Err(CausalityError::TypeConversionError(format!(
            "navigation world contextoid {id} is a scalar, not a series"
        ))),
    }
}

/// Read one scalar out of the navigation world.
pub fn scalar(context: &NavContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    match datum(context, id)? {
        NavDatum::Scalar(value) => Ok(value),
        NavDatum::Series(_) => Err(CausalityError::TypeConversionError(format!(
            "navigation world contextoid {id} is a series, not a scalar"
        ))),
    }
}

/// Read the value of one per-epoch series at epoch `idx`.
fn at(context: &NavContext, id: ContextoidId, idx: usize) -> Result<FloatType, CausalityError> {
    series(context, id)?.get(idx).copied().ok_or_else(|| {
        CausalityError::MissingParameter(format!("navigation world series {id} has no epoch {idx}"))
    })
}

/// The number of epochs in the navigation world.
pub fn epochs(context: &NavContext) -> Result<usize, CausalityError> {
    Ok(series(context, T_SEC)?.len())
}

/// The synthetic GNSS-denial indicator at epoch `idx`: a Gaussian bump centred on the world's
/// outage window — the signal degradation rising as the vehicle enters a GNSS-denied stretch
/// (jamming, an urban canyon, a tunnel, or terrain shadowing) and falling as it leaves.
pub fn denial_indicator(context: &NavContext, idx: usize) -> Result<FloatType, CausalityError> {
    Ok(denial_curve(context)?(idx))
}

/// The synthetic GNSS-denial indicator as a function of the epoch index, with the outage window
/// and the epoch count read from the world once.
pub fn denial_curve(context: &NavContext) -> Result<impl Fn(usize) -> FloatType, CausalityError> {
    let outage_lo = scalar(context, OUTAGE_START)?;
    let outage_hi = scalar(context, OUTAGE_END)?;
    let n = epochs(context)?;
    let centre = 0.5 * (outage_lo + outage_hi);
    let half_width = (0.5 * (outage_hi - outage_lo)).max(1e-6);
    Ok(move |idx: usize| {
        let frac = idx as f64 / n.saturating_sub(1) as f64;
        let z = (frac - centre) / half_width;
        (-(z * z)).exp()
    })
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

/// Build the processed epoch series from the real SP3 orbit + `.clk` clock series, each keyed by
/// its series contextoid id: radius and speed from the orbit, the nearest measured clock sample,
/// and the relativistic rate from the kernel. Fewer than three orbit epochs give empty series; an
/// epoch without a clock sample or a kernel failure is an error.
pub fn build_stream(
    mut orbits: Vec<OrbitData<FloatType>>,
    clocks: Vec<ClockData<FloatType>>,
) -> Result<[(ContextoidId, Vec<FloatType>); EPOCH_SERIES], PhysicsError> {
    orbits.sort_by_key(|o| o.timestamp());
    let n = orbits.len();
    if n < 3 {
        return Ok([
            T_SEC,
            DT,
            RADIUS_M,
            SPEED_MS,
            MEASURED_CLOCK_NS,
            RELATIVISTIC_RATE,
        ]
        .map(|id| (id, Vec::new())));
    }
    let mut t_secs = Vec::with_capacity(n);
    let mut dts = Vec::with_capacity(n);
    let mut radii_m = Vec::with_capacity(n);
    let mut speeds_ms = Vec::with_capacity(n);
    let mut measured_clocks_ns = Vec::with_capacity(n);
    let mut relativistic_rates = Vec::with_capacity(n);
    let t0 = orbits[0].timestamp();
    let billion = 1.0e9;

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
            .ok_or_else(|| {
                PhysicsError::CalculationError(format!("no clock sample for orbit epoch {i}"))
            })?;

        let relativistic_rate = relativistic_clock_drift_rate_kernel(radius_m, speed_ms, EARTH_GM)?;

        t_secs.push(t_sec);
        dts.push(dt);
        radii_m.push(radius_m);
        speeds_ms.push(speed_ms);
        measured_clocks_ns.push(measured_clock_ns);
        relativistic_rates.push(relativistic_rate);
    }
    Ok([
        (T_SEC, t_secs),
        (DT, dts),
        (RADIUS_M, radii_m),
        (SPEED_MS, speeds_ms),
        (MEASURED_CLOCK_NS, measured_clocks_ns),
        (RELATIVISTIC_RATE, relativistic_rates),
    ])
}

// ── CausalFlow stages ────────────────────────────────────────────────────────────────────────────

/// One tick: advance the INS error and carry both clocks one epoch. The value channel carries the INS
/// position error; the clocks and bookkeeping ride in `State`. Pure (no IO).
pub fn advance(
    value: CausalEffect<FloatType>,
    mut state: NavState,
    ctx: Option<NavContext>,
) -> NavProcess {
    let advanced = world(ctx.as_ref()).and_then(|world| {
        let prev_err = value
            .into_value()
            .ok_or_else(CausalityError::ValueNotAvailable)?;
        let idx = state.idx.min(epochs(world)?.saturating_sub(1));
        let dt = at(world, DT, idx)?;
        let relativistic_rate = at(world, RELATIVISTIC_RATE, idx)?;
        let mut logs = EffectLog::new();
        logs.add_entry(&format!(
            "epoch {:>3} t={:>6.0}s r={:.0}km v={:.0}m/s denial~{:.2}",
            idx,
            at(world, T_SEC, idx)?,
            at(world, RADIUS_M, idx)? / 1000.0,
            at(world, SPEED_MS, idx)?,
            denial_indicator(world, idx)?
        ));

        // INS error dynamics: a residual accelerometer bias integrates into velocity then position
        // error.
        state.ins_vel_err += state.ins_residual_bias * dt;
        let pos_err = prev_err + state.ins_vel_err * dt;

        // Both clocks always advance by their model rate; the GNSS-available arm disciplines them
        // back.
        let rel_rate_ns = relativistic_rate * 1.0e9; // ns per second
        state.carried_clock_ns += rel_rate_ns * dt;
        state.naive_clock_ns += state.last_rate_ns_per_s * dt;
        state.idx += 1;
        Ok((pos_err, logs))
    });
    match advanced {
        Ok((pos_err, logs)) => NavProcess::new(Ok(CausalEffect::value(pos_err)), state, ctx, logs),
        Err(e) => NavProcess::new(Err(e), state, ctx, EffectLog::new()),
    }
}

/// The grmhd `select_metric` pattern: compute the regime from a state indicator (the GNSS-denial
/// level) vs the world's threshold, set `state.gnss_denied`, and log the two regime changes
/// (entry / exit).
pub fn detect_regime(
    value: CausalEffect<FloatType>,
    mut state: NavState,
    ctx: Option<NavContext>,
) -> NavProcess {
    let read = world(ctx.as_ref()).and_then(|world| {
        let idx = state
            .idx
            .saturating_sub(1)
            .min(epochs(world)?.saturating_sub(1));
        Ok((
            denial_indicator(world, idx)?,
            scalar(world, BLACKOUT_THRESHOLD)?,
        ))
    });
    let (denial, threshold) = match read {
        Ok(read) => read,
        Err(e) => return NavProcess::new(Err(e), state, ctx, EffectLog::new()),
    };
    let denied = denial > threshold;

    let mut logs = EffectLog::new();
    if denied != state.prev_denied {
        state.regime_changes += 1;
        let (i, thr) = (denial, threshold);
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
/// the world's P-gain, substituted through `alternate_value` so the log records the override.
pub fn gps_fix(
    value: CausalEffect<FloatType>,
    state: NavState,
    ctx: Option<NavContext>,
) -> NavProcess {
    let fixed = world(ctx.as_ref()).and_then(|world| {
        let err = value
            .as_value()
            .copied()
            .ok_or_else(CausalityError::ValueNotAvailable)?;
        Ok(err * (1.0 - scalar(world, GPS_GAIN)?))
    });
    match fixed {
        Ok(fixed) => {
            NavProcess::new(Ok(value), state, ctx, EffectLog::new()).alternate_value(fixed)
        }
        Err(e) => NavProcess::new(Err(e), state, ctx, EffectLog::new()),
    }
}

/// State-side effect of a GNSS fix: calibrate the accelerometer bias and velocity error down, discipline
/// both carried clocks to the measured value, freeze the naive last-rate, and count the intervention.
/// The value passes through unchanged.
pub fn apply_fix(
    err: FloatType,
    state: &mut NavState,
    ctx: Option<&NavContext>,
) -> Result<FloatType, CausalityError> {
    let world = world(ctx)?;
    let idx = state
        .idx
        .saturating_sub(1)
        .min(epochs(world)?.saturating_sub(1));
    let measured_clock_ns = at(world, MEASURED_CLOCK_NS, idx)?;
    let relativistic_rate = at(world, RELATIVISTIC_RATE, idx)?;
    let bias_cal_gain = scalar(world, BIAS_CAL_GAIN)?;
    let gps_gain = scalar(world, GPS_GAIN)?;
    state.ins_residual_bias *= 1.0 - bias_cal_gain;
    state.ins_vel_err *= 1.0 - gps_gain;
    // Discipline both clocks to the real measured clock; latch the current relativistic rate for the
    // naive hold that takes over if GNSS is lost next tick.
    state.carried_clock_ns = measured_clock_ns;
    state.naive_clock_ns = measured_clock_ns;
    state.last_rate_ns_per_s = relativistic_rate * 1.0e9;
    state.correction_count += 1;
    Ok(err)
}

/// Harvest per-tick metrics (called each tick from the output stage). The value passes through
/// unchanged.
pub fn record_metrics(
    value: FloatType,
    state: &mut NavState,
    ctx: Option<&NavContext>,
) -> Result<FloatType, CausalityError> {
    let world = world(ctx)?;
    let err = value.abs();
    let idx = state
        .idx
        .saturating_sub(1)
        .min(epochs(world)?.saturating_sub(1));
    if state.gnss_denied {
        let cm = at(world, MEASURED_CLOCK_NS, idx)?;
        if err > state.peak_blackout_err {
            state.peak_blackout_err = err;
        }
        state.clock_carry_err_relativistic = (state.carried_clock_ns - cm).abs();
        state.clock_carry_err_naive = (state.naive_clock_ns - cm).abs();
    } else if state.regime_changes == 0 {
        state.pre_blackout_err = err;
    } else if state.regime_changes >= 2 {
        state.post_reacq_err = err;
    }
    state.final_err = err;
    Ok(value)
}

/// The state at the first epoch: zero INS error, the world's true (uncalibrated) accelerometer
/// bias loaded as the residual, clocks disciplined to the first measured sample.
fn initial_state(world: &NavContext) -> Result<NavState, CausalityError> {
    let first_clock = at(world, MEASURED_CLOCK_NS, 0)?;
    Ok(NavState {
        ins_residual_bias: scalar(world, ACCEL_BIAS)?,
        carried_clock_ns: first_clock,
        naive_clock_ns: first_clock,
        last_rate_ns_per_s: at(world, RELATIVISTIC_RATE, 0)? * 1.0e9,
        ..NavState::default()
    })
}

/// The initial process at the first epoch, carrying the navigation world in its `Context` channel.
pub fn initial_process(world: NavContext) -> NavProcess {
    match initial_state(&world) {
        Ok(state) => NavProcess::new(
            Ok(CausalEffect::value(0.0)),
            state,
            Some(world),
            EffectLog::new(),
        ),
        Err(e) => NavProcess::new(Err(e), NavState::default(), Some(world), EffectLog::new()),
    }
}
