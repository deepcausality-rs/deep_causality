/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The example-local physics stages both blackout examples compose: the freestream feeds, the
//! Sutton-Graves loads, the truth vehicle with its GNSS constellation, the commanded-bank
//! guidance, the weather telemetry accumulator, and the along-velocity witness a branch score
//! reads. These are corridor wiring, not library
//! physics; the library stages (chemistry, classifier, lift, navigation, the cybernetic gate)
//! live in `deep_causality_cfd`.

use super::FloatType;
use super::constants::{
    AIR_MEAN_MOLECULAR_MASS_KG, AIR_MOLECULE_DIAMETER_M, G0, GNSS_VAR, NOSE_RADIUS_M, RHO_REF,
    SUTTON_GRAVES_K,
};
use super::utils;
use deep_causality_algebra::Real;
use deep_causality_cfd::{CoupledField, PhysicsError, PhysicsStage, StepContext};
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
        let n_inf = utils::scalar0(field, "freestream_n");
        if n_inf <= utils::ft(0.0) {
            return Ok(());
        }
        let speed = utils::scalar0(field, "flight_speed");
        let d = utils::ft(AIR_MOLECULE_DIAMETER_M);
        let sigma = Real::sqrt(utils::ft(2.0)) * FloatType::pi() * d * d;
        let mfp = utils::ft(1.0) / (sigma * n_inf);
        let rho_inf = n_inf * utils::ft(AIR_MEAN_MOLECULAR_MASS_KG);
        let eas = speed * Real::sqrt(rho_inf / utils::ft(RHO_REF));
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
        let n_inf = utils::scalar0(field, "freestream_n");
        let speed = utils::scalar0(field, "flight_speed");
        let rho_inf = n_inf * utils::ft(AIR_MEAN_MOLECULAR_MASS_KG);
        let q = utils::ft(SUTTON_GRAVES_K)
            * Real::sqrt(rho_inf / utils::ft(NOSE_RADIUS_M))
            * speed
            * speed
            * speed;
        let a = field.aero_force().unwrap_or([utils::ft(0.0); 3]);
        let g = utils::norm3(a) / utils::ft(G0);
        field.set_scalar("heat_flux", Vec::from([q]));
        field.set_scalar("g_load", Vec::from([g]));
        Ok(())
    }
}

/// Deterministic receiver noise for the published fix: a golden-ratio low-discrepancy sequence
/// per axis, scaled so the per-axis variance is exactly `GNSS_VAR` (uniform on `±σ√3`).
/// Reproducible on every run, with no RNG dependency, consistent with the filter's `R`, and
/// computed in the working precision. `draw` selects one of infinitely many equally distributed
/// realizations by phase-shifting the sequence with the plastic constant; draw 0 is the
/// original sequence, and distinct draws are the Monte Carlo dimension of the dispersion table.
pub fn fix_noise(step: usize, draw: usize) -> [FloatType; 3] {
    const PHI: f64 = 0.618_033_988_749_894_9;
    const PLASTIC: f64 = 0.754_877_666_246_692_7;
    let amplitude = Real::sqrt(utils::ft(GNSS_VAR) * utils::ft(3.0));
    core::array::from_fn(|axis| {
        let stride = utils::ft(PHI) * (utils::ft(1.0) + utils::ft(0.37) * utils::ft(axis as f64));
        let phase = utils::ft(PLASTIC) * utils::ft(draw as f64)
            + utils::ft(0.29) * utils::ft((draw * axis) as f64);
        let x = (utils::ft(step as f64) + utils::ft(1.0)) * stride + phase;
        let u = x - x.floor();
        amplitude * (utils::ft(2.0) * u - utils::ft(1.0))
    })
}

/// The truth vehicle plus the GNSS constellation. Advances the true state with the true aero
/// force (drag and the bank-steered lift), then publishes the position fix with receiver noise
/// ([`fix_noise`]). The fix is always broadcast; whether the receiver can use it is the
/// corridor's denial gate, since the navigation stage folds it only when the classifier says
/// the link is up. The navigation drifts anyway: its IMU senses the same force through an
/// accelerometer bias. `noise_draw` selects the receiver-noise realization (0 by default).
#[derive(Debug, Clone, Copy, Default)]
pub struct TruthGnss {
    pub noise_draw: usize,
}

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
        let kick = field.aero_force().unwrap_or([utils::ft(0.0); 3]);
        let (r1, v1) = ks_strang_step(r, v, utils::ft(EARTH_GM), ctx.dt(), |_r, _v| kick)?;
        field.set_scalar(
            "truth_state",
            Vec::from([r1[0], r1[1], r1[2], v1[0], v1[1], v1[2]]),
        );
        let noise = fix_noise(ctx.step(), self.noise_draw);
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
        let bank = utils::scalar0(field, "commanded_bank");
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
            let step = utils::ft(ctx.step() as f64);
            let dwell = utils::scalar0(field, "wx_dwell_s") + ctx.dt();
            if field.scalar("wx_onset_step").is_none() {
                field.set_scalar("wx_onset_step", Vec::from([step]));
            }
            field.set_scalar("wx_last_denied_step", Vec::from([step]));
            field.set_scalar("wx_dwell_s", Vec::from([dwell]));

            let drift = match (field.scalar("nav_position"), field.scalar("truth_state")) {
                (Some(nav), Some(truth)) if nav.len() >= 3 && truth.len() >= 3 => {
                    utils::norm3(core::array::from_fn(|i| nav[i] - truth[i]))
                }
                _ => utils::ft(0.0),
            };
            if drift > utils::scalar0(field, "wx_drift_denied_max") {
                field.set_scalar("wx_drift_denied_max", Vec::from([drift]));
            }
        }
        let ne = field
            .scalar("n_e")
            .map(utils::peak)
            .unwrap_or_else(|| utils::ft(0.0));
        if ne > utils::scalar0(field, "wx_ne_max") {
            field.set_scalar("wx_ne_max", Vec::from([ne]));
        }
        let q = utils::scalar0(field, "heat_flux");
        if q > utils::scalar0(field, "wx_q_max") {
            field.set_scalar("wx_q_max", Vec::from([q]));
        }
        Ok(())
    }
}

/// The world-published fraction a frozen-drag foil holds the preserved-drag closure at.
///
/// A branch world publishes the fraction its trunk carried at the fork; absent, no foil is
/// accumulated. This is how the fork's drag state reaches a stage built before the fork existed.
pub const FROZEN_DRAG_FRACTION_FIELD: &str = "frozen_drag_fraction";

/// Publishes the along-velocity witnesses a branch score reads: the throttle the propulsion stages
/// actually flew, the axial deceleration the force channel actually carried, and two trajectory
/// velocity increments.
///
/// Composed **after** [`RetroThrust`](deep_causality_cfd::RetroThrust), so the force channel is the
/// summed one, and before the guidance, so `"realized_throttle"` is the value the propulsion stages
/// read this step rather than the command the guidance is about to write for the next.
///
/// Four scalars, all along `−v̂` and positive for deceleration:
///
/// * `"realized_throttle"` — the throttle channel the thrust and plume stages consumed. The
///   *commanded* throttle a world publishes is an input; the safety gate clamps it, so the two differ
///   whenever an axis binds and only the realized value describes the flight.
/// * `"axial_accel"` — `−(a · v̂)` from the summed force channel. This is the quantity a branch score
///   means by "the deceleration this branch realized", read where it is produced.
/// * `"dv_actual"` — `Σ axial_accel · Δt`, the velocity the descent actually shed.
/// * `"dv_frozen"` — the same thrust schedule with the preserved-drag fraction held at the fork's
///   value: `Σ (a_thrust + f_fork · a_drag) · Δt`. A foil that differs from `dv_actual` only through
///   the drag closure, so their separation isolates the coupling. Accumulated only while a world
///   publishes [`FROZEN_DRAG_FRACTION_FIELD`].
///
/// The drag term is formed from the flown dynamic pressure and the vehicle's own ballistic bundle
/// (`a_drag = q∞ · C_d·A/m`), which is what the lift stage put on the channel — not from a separate
/// drag constant, which would be a second source of truth for a quantity the run already computes.
#[derive(Debug, Clone, Copy)]
pub struct AxialWitness {
    thrust: FloatType,
    cd_area_over_mass: FloatType,
}

impl AxialWitness {
    /// A witness over a vehicle with full-throttle thrust `thrust` (N) and ballistic bundle
    /// `cd_area_over_mass` (m²·kg⁻¹) — the same bundle the lift stage flies.
    pub fn new(thrust: FloatType, cd_area_over_mass: FloatType) -> Self {
        Self {
            thrust,
            cd_area_over_mass,
        }
    }
}

impl PhysicsStage<2, FloatType> for AxialWitness {
    fn apply(
        &self,
        ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let Some(truth) = field.scalar("truth_state") else {
            return Ok(());
        };
        if truth.len() < 6 {
            return Ok(());
        }
        let v = [truth[3], truth[4], truth[5]];
        let speed = utils::norm3(v);
        if speed <= utils::ft(0.0) {
            return Ok(());
        }
        let v_hat: [FloatType; 3] = core::array::from_fn(|i| v[i] / speed);

        // The realized throttle: what the propulsion stages consumed, not what a world commanded.
        let throttle = field.throttle_action().unwrap_or_else(|| utils::ft(0.0));
        field.set_scalar("realized_throttle", Vec::from([throttle]));

        // The realized axial deceleration, read off the summed force channel.
        let a = field.aero_force().unwrap_or([utils::ft(0.0); 3]);
        let along = a[0] * v_hat[0] + a[1] * v_hat[1] + a[2] * v_hat[2];
        let axial = utils::ft(0.0) - along;
        field.set_scalar("axial_accel", Vec::from([axial]));

        let dv_actual = utils::scalar0(field, "dv_actual") + axial * ctx.dt();
        field.set_scalar("dv_actual", Vec::from([dv_actual]));

        // The frozen-drag foil, accumulated only where a world names the fraction to freeze at.
        if let Some(f_fork) = field
            .scalar(FROZEN_DRAG_FRACTION_FIELD)
            .and_then(|s| s.first().copied())
        {
            let mass = utils::scalar0(field, "mass");
            let q_inf = utils::scalar0(field, "q_inf");
            if mass > utils::ft(0.0) {
                // The aerodynamic drag the lift stage produced, before the plume closure scaled it.
                let a_drag = q_inf * self.cd_area_over_mass;
                let a_thrust = throttle * self.thrust / mass;
                let frozen = a_thrust + f_fork * a_drag;
                let dv_frozen = utils::scalar0(field, "dv_frozen") + frozen * ctx.dt();
                field.set_scalar("dv_frozen", Vec::from([dv_frozen]));
            }
        }
        Ok(())
    }
}
