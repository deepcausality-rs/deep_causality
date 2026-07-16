/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The weather-table model: the weather cases, the baseline and per-condition worlds, the thermal
//! bias departure, the ensemble reduction that collapses a condition's draw set into one table
//! row, and the two gating sequences (the table certification checklist and the caller's
//! wall-clock budget). The shared stages, constants, and world builder live in
//! `avionics_examples::shared`.

use crate::FloatType;
use crate::constants::{
    COLD_DRIFT_FACTOR_MIN, DRIFT_SIGNIFICANCE_SIGMA, IMU_THERMAL_COEFF_PER_K, MC_DRAWS,
    MIN_ONSET_SPREAD_S, REACQ_ERR_MAX_M, STEPS, WALL_CLOCK_BUDGET_S, WEATHER,
};
use avionics_examples::shared::constants::DT_FLIGHT;
use avionics_examples::shared::utils::{ft, norm3};
use avionics_examples::shared::world;
use deep_causality_cfd::{
    CompressibleMarchConfig, GateSeq, PhysicsError, Report, StudyView, TableRow,
};
use std::path::PathBuf;

// ── The case axis: weather conditions ─────────────────────────────────────────────────────────

/// One weather condition of the dispersion table: the temperature offset and density scale that
/// define the counterfactual atmosphere. These are `f64` specification inputs (like the atmosphere
/// table itself); the computed flow quantities the table reports are [`FloatType`].
#[derive(Debug, Clone)]
pub struct WeatherCase {
    pub name: &'static str,
    pub d_temp: f64,
    pub rho_scale: f64,
}

/// The weather cases (the study's case axis) from the tuned condition table.
pub fn weather_cases() -> Vec<WeatherCase> {
    WEATHER
        .iter()
        .map(|&(name, d_temp, rho_scale)| WeatherCase {
            name,
            d_temp,
            rho_scale,
        })
        .collect()
}

/// The validated baseline world: the corridor's standard-day descent, built once. The study's
/// counterfactual atmospheres alternate from this.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn standard_day() -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world_cfg("standard_day", 0.0, 1.0)
}

/// A weather world: the baseline descent with the condition's temperature offset and density scale
/// applied, carrying its thermal bias departure as a world-published constant for the provenance
/// trail.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn weather_world(
    case: &WeatherCase,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world_cfg(case.name, case.d_temp, case.rho_scale)
}

fn world_cfg(
    name: &'static str,
    d_temp: f64,
    rho_scale: f64,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world::descent_world(
        name,
        world::weather_atmosphere(d_temp, rho_scale),
        STEPS,
        &[("wx_bias_departure", ft(bias_departure(d_temp)))],
    )
}

/// The thermal-departure factor of the accelerometer bias for a condition `d_temp` away from the
/// calibration point: `1 + K_T * |dT|`. The departure grows in either direction because the unit
/// was calibrated at standard conditions, not because cold is intrinsically worse.
pub fn bias_departure(d_temp: f64) -> f64 {
    1.0 + IMU_THERMAL_COEFF_PER_K * d_temp.abs()
}

// ── The ensemble reduction: one condition's draw set → one row ─────────────────────────────────

/// One row of the dispersion table: the flow and window metrics from the reference draw (the
/// receiver noise does not touch the flow, chemistry, or truth trajectory, so those columns are
/// draw-invariant), plus Monte Carlo statistics of the navigation metrics over all draws.
#[derive(Debug, Clone)]
pub struct WorldRow {
    pub name: &'static str,
    pub d_temp: f64,
    pub rho_scale: f64,
    /// The thermal-departure factor the IMU flew.
    pub bias_departure: f64,
    pub errored: bool,
    pub has_alternation_marker: bool,
    /// Blackout onset and exit, seconds from the entry interface (0 when never denied).
    pub onset_s: FloatType,
    pub exit_s: FloatType,
    pub dwell_s: FloatType,
    pub ne_max: FloatType,
    pub q_max: FloatType,
    /// Maximum dead-reckoning drift while denied: mean and sample standard deviation over the
    /// receiver-noise draws, m.
    pub drift_mean_m: FloatType,
    pub drift_sd_m: FloatType,
    /// Terminal navigation error after reacquisition: mean over the draws and the worst draw, m.
    pub terminal_mean_m: FloatType,
    pub terminal_max_m: FloatType,
}

impl TableRow for WorldRow {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[
        ("d_temp", "K"),
        ("rho_scale", "-"),
        ("bias_departure", "-"),
        ("onset", "s"),
        ("exit", "s"),
        ("dwell", "s"),
        ("ne_max", "m^-3"),
        ("q_max", "W/m2"),
        ("drift_mean", "m"),
        ("drift_sd", "m"),
        ("terminal_mean", "m"),
        ("terminal_max", "m"),
    ];
    fn cells(&self) -> Vec<FloatType> {
        vec![
            ft(self.d_temp),
            ft(self.rho_scale),
            ft(self.bias_departure),
            self.onset_s,
            self.exit_s,
            self.dwell_s,
            self.ne_max,
            self.q_max,
            self.drift_mean_m,
            self.drift_sd_m,
            self.terminal_mean_m,
            self.terminal_max_m,
        ]
    }
}

/// The first value of a report's `final_<scalar>` series (the coupling's telemetry, exposed on the
/// report), or `0` when absent.
fn scalar0(report: &Report<FloatType>, name: &str) -> FloatType {
    report
        .series(name)
        .and_then(|s| s.first().copied())
        .unwrap_or_else(|| ft(0.0))
}

/// The navigation metrics of one draw: (maximum drift while denied, terminal error), read from the
/// report's terminal witnesses.
fn draw_metrics(report: &Report<FloatType>) -> (FloatType, FloatType) {
    let terminal = match (
        report.series("final_nav_position"),
        report.series("final_truth_state"),
    ) {
        (Some(nav), Some(truth)) if nav.len() >= 3 && truth.len() >= 3 => {
            norm3(core::array::from_fn(|i| nav[i] - truth[i]))
        }
        _ => ft(f64::NAN),
    };
    (scalar0(report, "final_wx_drift_denied_max"), terminal)
}

/// Mean and sample standard deviation of a slice.
fn mean_sd(xs: &[FloatType]) -> (FloatType, FloatType) {
    let n = ft(xs.len() as f64);
    let mean = xs.iter().copied().sum::<FloatType>() / n;
    if xs.len() < 2 {
        return (mean, ft(0.0));
    }
    let var = xs
        .iter()
        .map(|&x| (x - mean) * (x - mean))
        .sum::<FloatType>()
        / (n - ft(1.0));
    (mean, deep_causality_algebra::Real::sqrt(var))
}

/// The ensemble reduction: one condition's finished draw set (the reference draw first) collapses
/// to one table row — flow/window metrics from the reference draw, navigation statistics over all
/// draws.
///
/// # Errors
/// Never fails today; returns `Result` to fit the `reduce_ensemble` seam.
pub fn world_row(
    case: &WeatherCase,
    draws: &[Report<FloatType>],
) -> Result<WorldRow, PhysicsError> {
    let reference = &draws[0];
    let step_s = |scalar: &str| scalar0(reference, scalar) * ft(DT_FLIGHT);

    let metrics: Vec<(FloatType, FloatType)> = draws.iter().map(draw_metrics).collect();
    let drifts: Vec<FloatType> = metrics.iter().map(|&(d, _)| d).collect();
    let terminals: Vec<FloatType> = metrics.iter().map(|&(_, t)| t).collect();
    let (drift_mean_m, drift_sd_m) = mean_sd(&drifts);
    let (terminal_mean_m, _) = mean_sd(&terminals);
    // NaN-honest maximum: a draw with a missing truth state reports NaN, and the row's aggregate
    // must stay NaN so the reacquisition gate fails instead of scoring a perfect 0.
    let terminal_max_m = terminals
        .iter()
        .copied()
        .reduce(|a, x| if x.is_nan() || x > a { x } else { a })
        .unwrap_or_else(|| ft(f64::NAN));

    Ok(WorldRow {
        name: case.name,
        d_temp: case.d_temp,
        rho_scale: case.rho_scale,
        bias_departure: bias_departure(case.d_temp),
        errored: false, // a report that exists marched to completion; an error short-circuits march_for
        has_alternation_marker: reference
            .effect_log()
            .map(|l| format!("{l}").contains("!!ContextAlternation!!"))
            .unwrap_or(false),
        onset_s: step_s("final_wx_onset_step"),
        exit_s: step_s("final_wx_last_denied_step"),
        dwell_s: scalar0(reference, "final_wx_dwell_s"),
        ne_max: scalar0(reference, "final_wx_ne_max"),
        q_max: scalar0(reference, "final_wx_q_max"),
        drift_mean_m,
        drift_sd_m,
        terminal_mean_m,
        terminal_max_m,
    })
}

// ── The table certification gating sequence (over the world rows) ─────────────────────────────

/// The dispersion table's certification checklist as one reviewable value: integrity, the
/// counterfactual audit trail, flow-resolved windows everywhere, weather moving the window, the
/// cold INS effect (mean and statistically resolved), and reacquisition in every draw.
pub fn weather_gates() -> GateSeq<WorldRow> {
    GateSeq::new("weather-dispersion table")
        .gate("(0) table integrity", gate_integrity)
        .gate("(1) counterfactual audit trail", gate_markers)
        .gate("(2) flow-resolved windows in every weather", gate_windows)
        .gate("(3) weather moves the blackout window", gate_window_spread)
        .gate(
            "(4) the INS does not behave as assumed in the cold",
            gate_cold_drift,
        )
        .gate(
            "(4b) the cold effect is statistically resolved",
            gate_cold_sigma,
        )
        .gate("(5) every weather reacquires, in every draw", gate_reacq)
}

fn gate_integrity(v: &StudyView<'_, WorldRow>) -> (bool, String) {
    (
        v.rows().iter().all(|r| !r.errored),
        "all six worlds completed without a captured step error".into(),
    )
}

fn gate_markers(v: &StudyView<'_, WorldRow>) -> (bool, String) {
    (
        v.rows().iter().skip(1).all(|r| r.has_alternation_marker),
        "every dispersion world carries the !!ContextAlternation!! marker naming its baseline"
            .into(),
    )
}

fn gate_windows(v: &StudyView<'_, WorldRow>) -> (bool, String) {
    let horizon_s = ft((STEPS - 1) as f64 * DT_FLIGHT);
    (
        v.rows().iter().all(|r| {
            r.onset_s > ft(0.0)
                && r.dwell_s > ft(0.0)
                && r.exit_s >= r.onset_s
                && r.exit_s < horizon_s
        }),
        "each world found onset, a nonzero dwell, and a recovered link before the horizon".into(),
    )
}

fn gate_window_spread(v: &StudyView<'_, WorldRow>) -> (bool, String) {
    let onset_hi = v
        .rows()
        .iter()
        .map(|r| r.onset_s)
        .fold(ft(f64::MIN), FloatType::max);
    let onset_lo = v
        .rows()
        .iter()
        .map(|r| r.onset_s)
        .fold(ft(f64::MAX), FloatType::min);
    let dwell_hi = v
        .rows()
        .iter()
        .map(|r| r.dwell_s)
        .fold(ft(f64::MIN), FloatType::max);
    let dwell_lo = v
        .rows()
        .iter()
        .map(|r| r.dwell_s)
        .fold(ft(f64::MAX), FloatType::min);
    (
        onset_hi - onset_lo >= ft(MIN_ONSET_SPREAD_S),
        format!(
            "onset spread {:.1} s across the table (gate requires {:.0} s); dwell spread {:.1} s",
            onset_hi - onset_lo,
            MIN_ONSET_SPREAD_S,
            dwell_hi - dwell_lo,
        ),
    )
}

fn gate_cold_drift(v: &StudyView<'_, WorldRow>) -> (bool, String) {
    let standard = &v.rows()[0];
    let polar = v
        .rows()
        .iter()
        .find(|r| r.name == "polar_winter")
        .unwrap_or(standard);
    (
        polar.drift_mean_m >= standard.drift_mean_m * ft(COLD_DRIFT_FACTOR_MIN),
        format!(
            "polar-winter mean blackout drift {:.2} m vs standard-day {:.2} m ({:.2}x; gate \
             requires {:.1}x from the thermal bias departure and the widened window)",
            polar.drift_mean_m,
            standard.drift_mean_m,
            polar.drift_mean_m / standard.drift_mean_m,
            COLD_DRIFT_FACTOR_MIN,
        ),
    )
}

fn gate_cold_sigma(v: &StudyView<'_, WorldRow>) -> (bool, String) {
    let standard = &v.rows()[0];
    let polar = v
        .rows()
        .iter()
        .find(|r| r.name == "polar_winter")
        .unwrap_or(standard);
    let combined_sd = deep_causality_algebra::Real::sqrt(
        polar.drift_sd_m * polar.drift_sd_m + standard.drift_sd_m * standard.drift_sd_m,
    );
    let separation = polar.drift_mean_m - standard.drift_mean_m;
    (
        combined_sd.is_finite()
            && combined_sd > ft(0.0)
            && separation > combined_sd * ft(DRIFT_SIGNIFICANCE_SIGMA),
        format!(
            "polar-standard separation {:.2} m vs combined sigma {:.2} m ({:.1} sigma; gate \
             requires {:.0})",
            separation,
            combined_sd,
            separation / combined_sd,
            DRIFT_SIGNIFICANCE_SIGMA,
        ),
    )
}

fn gate_reacq(v: &StudyView<'_, WorldRow>) -> (bool, String) {
    (
        v.rows()
            .iter()
            .all(|r| r.terminal_max_m < ft(REACQ_ERR_MAX_M)),
        format!(
            "worst-draw terminal navigation error under {:.1} m across all {} descents",
            REACQ_ERR_MAX_M,
            WEATHER.len() * MC_DRAWS,
        ),
    )
}

// ── The caller's wall-clock gate (the study cannot see wall-clock; it times the whole program) ─

/// The wall-clock budget gate, checked by the caller over the elapsed seconds and merged into the
/// table verdict. Reads the single elapsed-seconds "row" through [`StudyView::of`].
pub fn runtime_gates() -> GateSeq<FloatType> {
    GateSeq::new("weather runtime").gate("(6) wall-clock budget", gate_wall_clock)
}

fn gate_wall_clock(v: &StudyView<'_, FloatType>) -> (bool, String) {
    let elapsed_s = v.rows()[0];
    (
        elapsed_s < ft(WALL_CLOCK_BUDGET_S),
        format!(
            "{elapsed_s:.1} s for the whole six-world table (budget {:.0} s)",
            WALL_CLOCK_BUDGET_S,
        ),
    )
}

pub fn get_audit_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cfd/plasma_blackout/weather/audit")
}

/// Where the dispersion table is recorded (the campaign's `record` seam).
pub fn get_table_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cfd/plasma_blackout/weather/weather_table.csv")
}
