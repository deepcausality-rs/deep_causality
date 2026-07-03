/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The weather-table model: one descent world per condition (through the shared blackout
//! machinery), the thermal bias departure, and the table-row extraction from a finished world.
//! The shared stages, constants, and world builder live in `avionics_examples::blackout`.

use crate::FloatType;
use crate::constants::{IMU_THERMAL_COEFF_PER_K, STEPS};
use avionics_examples::blackout::constants::DT_FLIGHT;
use avionics_examples::blackout::{utils, world};
use deep_causality_cfd::{CompressibleMarchConfig, CompressiblePause, PhysicsError};

/// A weather world: the baseline descent with the condition's temperature offset and density
/// scale applied, carrying its thermal bias departure as a world-published constant for the
/// provenance trail.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn weather_world(
    name: &'static str,
    d_temp: f64,
    rho_scale: f64,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world::descent_world(
        name,
        world::weather_atmosphere(d_temp, rho_scale),
        STEPS,
        &[("wx_bias_departure", utils::ft(bias_departure(d_temp)))],
    )
}

/// The thermal-departure factor of the accelerometer bias for a condition `d_temp` away from
/// the calibration point: `1 + K_T * |dT|`. The departure grows in either direction because the
/// unit was calibrated at standard conditions, not because cold is intrinsically worse.
pub fn bias_departure(d_temp: f64) -> f64 {
    1.0 + IMU_THERMAL_COEFF_PER_K * d_temp.abs()
}

/// One row of the dispersion table: the flow and window metrics from the reference draw (the
/// receiver noise does not touch the flow, chemistry, or truth trajectory, so those columns are
/// draw-invariant), plus Monte Carlo statistics of the navigation metrics over all draws.
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

/// The navigation metrics of one draw: (maximum drift while denied, terminal error).
pub fn draw_metrics<S>(pause: &CompressiblePause<'_, FloatType, S>) -> (FloatType, FloatType) {
    let field = pause.field();
    let truth = field.scalar("truth_state").unwrap_or(&[]);
    let terminal = match (field.nav(), truth.len() >= 3) {
        (Some(engine), true) => {
            let p = engine.position();
            utils::norm3(core::array::from_fn(|i| p[i] - truth[i]))
        }
        _ => utils::ft(f64::NAN),
    };
    (utils::scalar0(field, "wx_drift_denied_max"), terminal)
}

/// Mean and sample standard deviation of a slice.
pub fn mean_sd(xs: &[FloatType]) -> (FloatType, FloatType) {
    let n = utils::ft(xs.len() as f64);
    let mean = xs.iter().copied().sum::<FloatType>() / n;
    if xs.len() < 2 {
        return (mean, utils::ft(0.0));
    }
    let var = xs
        .iter()
        .map(|&x| (x - mean) * (x - mean))
        .sum::<FloatType>()
        / (n - utils::ft(1.0));
    (mean, deep_causality_num::Real::sqrt(var))
}

/// Extract the table row from one condition's finished draws (the reference draw first).
pub fn world_row<S>(
    name: &'static str,
    d_temp: f64,
    rho_scale: f64,
    draws: &[CompressiblePause<'_, FloatType, S>],
) -> WorldRow {
    let reference = &draws[0];
    let field = reference.field();
    let step_s = |scalar: &str| utils::scalar0(field, scalar) * utils::ft(DT_FLIGHT);

    let metrics: Vec<(FloatType, FloatType)> = draws.iter().map(draw_metrics).collect();
    let drifts: Vec<FloatType> = metrics.iter().map(|&(d, _)| d).collect();
    let terminals: Vec<FloatType> = metrics.iter().map(|&(_, t)| t).collect();
    let (drift_mean_m, drift_sd_m) = mean_sd(&drifts);
    let (terminal_mean_m, _) = mean_sd(&terminals);
    // NaN-honest maximum: a draw with a missing truth state reports NaN, and the row's
    // aggregate must stay NaN so the reacquisition gate fails instead of scoring a perfect 0.
    let terminal_max_m = terminals
        .iter()
        .copied()
        .reduce(|a, x| if x.is_nan() || x > a { x } else { a })
        .unwrap_or_else(|| utils::ft(f64::NAN));

    WorldRow {
        name,
        d_temp,
        rho_scale,
        bias_departure: bias_departure(d_temp),
        errored: draws.iter().any(|p| p.error().is_some()),
        has_alternation_marker: format!("{}", field.log()).contains("!!ContextAlternation!!"),
        onset_s: step_s("wx_onset_step"),
        exit_s: step_s("wx_last_denied_step"),
        dwell_s: utils::scalar0(field, "wx_dwell_s"),
        ne_max: utils::scalar0(field, "wx_ne_max"),
        q_max: utils::scalar0(field, "wx_q_max"),
        drift_mean_m,
        drift_sd_m,
        terminal_mean_m,
        terminal_max_m,
    }
}
