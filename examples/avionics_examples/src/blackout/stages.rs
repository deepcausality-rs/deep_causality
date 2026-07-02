/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The example-local physics stages both blackout examples compose: the freestream feeds, the
//! Sutton-Graves loads, the truth vehicle with its GNSS constellation, the commanded-bank
//! guidance, and the weather telemetry accumulator. These are corridor wiring, not library
//! physics; the library stages (chemistry, classifier, lift, navigation, the cybernetic gate)
//! live in `deep_causality_cfd`.

use super::FloatType;
use super::constants::{
    AIR_MEAN_MOLECULAR_MASS_KG, AIR_MOLECULE_DIAMETER_M, G0, GNSS_VAR, NOSE_RADIUS_M, RHO_REF,
    SUTTON_GRAVES_K,
};
use super::support;
use deep_causality_cfd::{CoupledField, PhysicsError, PhysicsStage, StepContext};
use deep_causality_num::Real;
use deep_causality_physics::{EARTH_GM, ks_strang_step};

/// Derives the per-step freestream feeds from the carrier's published flight scalars:
///
/// * `"mean_free_path"`: the freestream Knudsen driver, `λ = 1/(√2·π·d²·n_∞)` (hard-sphere air).
/// * `"equivalent_airspeed"`: `EAS = V·√(ρ_∞/ρ_ref)`, so the lift stage's fixed-density dynamic
///   pressure `½·ρ_ref·EAS²` equals the true `½·ρ_∞·V²` at every altitude.
///
/// A no-op until the carrier has published the flight scalars.
#[derive(Debug, Clone, Copy)]
pub struct FreestreamFeeds;

impl PhysicsStage<2, FloatType> for FreestreamFeeds {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let n_inf = support::scalar0(field, "freestream_n");
        if n_inf <= support::ft(0.0) {
            return Ok(());
        }
        let speed = support::scalar0(field, "flight_speed");
        let d = support::ft(AIR_MOLECULE_DIAMETER_M);
        let sigma = Real::sqrt(support::ft(2.0)) * FloatType::pi() * d * d;
        let mfp = support::ft(1.0) / (sigma * n_inf);
        let rho_inf = n_inf * support::ft(AIR_MEAN_MOLECULAR_MASS_KG);
        let eas = speed * Real::sqrt(rho_inf / support::ft(RHO_REF));
        field.set_scalar("mean_free_path", Vec::from([mfp]));
        field.set_scalar("equivalent_airspeed", Vec::from([eas]));
        Ok(())
    }
}

/// Publishes the sensed flight loads the envelope gate reads: the Sutton-Graves stagnation
/// heating `q = k·√(ρ_∞/R_n)·V³` from the **evolved schedule density and flight speed**, and
/// the g-load off the aero channel.
#[derive(Debug, Clone, Copy)]
pub struct SuttonGravesLoads;

impl PhysicsStage<2, FloatType> for SuttonGravesLoads {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let n_inf = support::scalar0(field, "freestream_n");
        let speed = support::scalar0(field, "flight_speed");
        let rho_inf = n_inf * support::ft(AIR_MEAN_MOLECULAR_MASS_KG);
        let q = support::ft(SUTTON_GRAVES_K)
            * Real::sqrt(rho_inf / support::ft(NOSE_RADIUS_M))
            * speed
            * speed
            * speed;
        let a = field.aero_force().unwrap_or([support::ft(0.0); 3]);
        let g = support::norm3(a) / support::ft(G0);
        field.set_scalar("heat_flux", Vec::from([q]));
        field.set_scalar("g_load", Vec::from([g]));
        Ok(())
    }
}

/// Deterministic receiver noise for the published fix: a golden-ratio low-discrepancy sequence
/// per axis, scaled so the per-axis variance is exactly `GNSS_VAR` (uniform on `±σ√3`).
/// Reproducible on every run, with no RNG dependency, consistent with the filter's `R`, and
/// computed in the working precision.
pub fn fix_noise(step: usize) -> [FloatType; 3] {
    const PHI: f64 = 0.618_033_988_749_894_9;
    let amplitude = Real::sqrt(support::ft(GNSS_VAR) * support::ft(3.0));
    core::array::from_fn(|axis| {
        let stride =
            support::ft(PHI) * (support::ft(1.0) + support::ft(0.37) * support::ft(axis as f64));
        let x = (support::ft(step as f64) + support::ft(1.0)) * stride;
        let u = x - x.floor();
        amplitude * (support::ft(2.0) * u - support::ft(1.0))
    })
}

/// The truth vehicle plus the GNSS constellation. Advances the true state with the true aero
/// force (drag and the bank-steered lift), then publishes the position fix with receiver noise
/// ([`fix_noise`]). The fix is always broadcast; whether the receiver can use it is the
/// corridor's denial gate, since the navigation stage folds it only when the classifier says
/// the link is up. The navigation drifts anyway: its IMU senses the same force through an
/// accelerometer bias.
#[derive(Debug, Clone, Copy)]
pub struct TruthGnss;

impl PhysicsStage<2, FloatType> for TruthGnss {
    fn apply(
        &self,
        ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let Some(state) = field.scalar("truth_state") else {
            return Ok(());
        };
        let r = [state[0], state[1], state[2]];
        let v = [state[3], state[4], state[5]];
        let kick = field.aero_force().unwrap_or([support::ft(0.0); 3]);
        let (r1, v1) = ks_strang_step(r, v, support::ft(EARTH_GM), ctx.dt(), |_r, _v| kick)?;
        field.set_scalar(
            "truth_state",
            Vec::from([r1[0], r1[1], r1[2], v1[0], v1[1], v1[2]]),
        );
        let noise = fix_noise(ctx.step());
        field.set_scalar(
            "gnss_fix",
            Vec::from([r1[0] + noise[0], r1[1] + noise[1], r1[2] + noise[2]]),
        );
        Ok(())
    }
}

/// The guidance law: command the world's published bank (`"commanded_bank"`, the constant each
/// counterfactual world carries; zero when absent, i.e. ballistic). The raw command lands in
/// the control channel; the cybernetic gate clamps it into the envelope, and the lift stage
/// flies the clamped value on the next step (the one-step actuation lag).
#[derive(Debug, Clone, Copy)]
pub struct CommandedBank;

impl PhysicsStage<2, FloatType> for CommandedBank {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let bank = support::scalar0(field, "commanded_bank");
        field.set_control_action(bank);
        Ok(())
    }
}

/// Accumulates per-descent telemetry into named field scalars, one update per step: the
/// blackout window (`wx_onset_step`, `wx_last_denied_step`, `wx_dwell_s`), the maximum
/// dead-reckoning drift while the link is denied (`wx_drift_denied_max`), and the peak evolved
/// electron density and sensed heating (`wx_ne_max`, `wx_q_max`). Composed after the navigation
/// stage, so it reads the same step's solution.
#[derive(Debug, Clone, Copy)]
pub struct WeatherTelemetry;

impl PhysicsStage<2, FloatType> for WeatherTelemetry {
    fn apply(
        &self,
        ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let denied = field.regime().map(|r| r.gnss_denied).unwrap_or(false);
        if denied {
            let step = support::ft(ctx.step() as f64);
            let dwell = support::scalar0(field, "wx_dwell_s") + ctx.dt();
            if field.scalar("wx_onset_step").is_none() {
                field.set_scalar("wx_onset_step", Vec::from([step]));
            }
            field.set_scalar("wx_last_denied_step", Vec::from([step]));
            field.set_scalar("wx_dwell_s", Vec::from([dwell]));

            let drift = match (field.scalar("nav_position"), field.scalar("truth_state")) {
                (Some(nav), Some(truth)) if nav.len() >= 3 && truth.len() >= 3 => {
                    support::norm3(core::array::from_fn(|i| nav[i] - truth[i]))
                }
                _ => support::ft(0.0),
            };
            if drift > support::scalar0(field, "wx_drift_denied_max") {
                field.set_scalar("wx_drift_denied_max", Vec::from([drift]));
            }
        }
        let ne = field
            .scalar("n_e")
            .map(support::peak)
            .unwrap_or_else(|| support::ft(0.0));
        if ne > support::scalar0(field, "wx_ne_max") {
            field.set_scalar("wx_ne_max", Vec::from([ne]));
        }
        let q = support::scalar0(field, "heat_flux");
        if q > support::scalar0(field, "wx_q_max") {
            field.set_scalar("wx_q_max", Vec::from([q]));
        }
        Ok(())
    }
}
