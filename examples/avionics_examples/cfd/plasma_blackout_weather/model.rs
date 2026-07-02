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
use avionics_examples::blackout::{support, world};
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
        &[("wx_bias_departure", support::ft(bias_departure(d_temp)))],
    )
}

/// The thermal-departure factor of the accelerometer bias for a condition `d_temp` away from
/// the calibration point: `1 + K_T * |dT|`. The departure grows in either direction because the
/// unit was calibrated at standard conditions, not because cold is intrinsically worse.
pub fn bias_departure(d_temp: f64) -> f64 {
    1.0 + IMU_THERMAL_COEFF_PER_K * d_temp.abs()
}

/// One row of the dispersion table, extracted from a finished world's carried field.
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
    /// Maximum dead-reckoning drift while the link was denied, m.
    pub drift_denied_max_m: FloatType,
    /// Terminal navigation error after reacquisition, m.
    pub terminal_err_m: FloatType,
}

/// Extract the table row from a paused (finished) world.
pub fn world_row<S>(
    name: &'static str,
    d_temp: f64,
    rho_scale: f64,
    pause: &CompressiblePause<'_, FloatType, S>,
) -> WorldRow {
    let field = pause.field();
    let step_s = |scalar: &str| support::scalar0(field, scalar) * support::ft(DT_FLIGHT);
    let truth = field.scalar("truth_state").unwrap_or(&[]);
    let terminal_err_m = match (field.nav(), truth.len() >= 3) {
        (Some(engine), true) => {
            let p = engine.position();
            support::norm3(core::array::from_fn(|i| p[i] - truth[i]))
        }
        _ => support::ft(f64::NAN),
    };
    WorldRow {
        name,
        d_temp,
        rho_scale,
        bias_departure: bias_departure(d_temp),
        errored: pause.error().is_some(),
        has_alternation_marker: format!("{}", field.log()).contains("!!ContextAlternation!!"),
        onset_s: step_s("wx_onset_step"),
        exit_s: step_s("wx_last_denied_step"),
        dwell_s: support::scalar0(field, "wx_dwell_s"),
        ne_max: support::scalar0(field, "wx_ne_max"),
        q_max: support::scalar0(field, "wx_q_max"),
        drift_denied_max_m: support::scalar0(field, "wx_drift_denied_max"),
        terminal_err_m,
    }
}
