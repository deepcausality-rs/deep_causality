/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The counterfactual branch vocabulary (\[5\]) and the 3-DOF bank-steered aero producer (\[5b\]).

use super::super::coupling::{CoupledField, PhysicsStage, StepContext};
use super::{cross3, dot3, norm3, peak, scale3};
use crate::CfdScalar;
use deep_causality_physics::PhysicsError;

/// The outcome of one counterfactual bank-angle branch — the four scores the corridor compares
/// across candidate bank schedules: peak heat flux, integrated thermal load, terminal miss distance,
/// and total comms-blackout dwell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BranchOutcome<R: CfdScalar> {
    /// The (constant) commanded bank angle for this branch (rad).
    pub bank_angle: R,
    /// Peak wall heat flux over the window (W·m⁻²).
    pub peak_heat_flux: R,
    /// Time-integrated heat load over the window (J·m⁻²).
    pub thermal_load: R,
    /// Terminal miss distance from the aim point (m).
    pub miss_distance: R,
    /// Total time the comms link was denied over the window (s).
    pub blackout_dwell: R,
}

/// A predict-only reducer for one bank-angle branch: fold each rolled-out step's instantaneous heat
/// flux, comms-denial flag, and `dt` with [`observe`](Self::observe), then close with the terminal
/// miss distance in [`finish`](Self::finish). The alternate-world rollout driver (Stage 4's
/// `run_coupled` over an alternated context) feeds this; keeping the fold here makes the branch
/// scoring a small, exhaustively-tested unit independent of the march machinery.
#[derive(Debug, Clone, Copy)]
pub struct BranchAccumulator<R: CfdScalar> {
    bank_angle: R,
    peak_heat_flux: R,
    thermal_load: R,
    blackout_dwell: R,
}

impl<R: CfdScalar> BranchAccumulator<R> {
    /// Begin accumulating the branch flown at constant bank angle `bank_angle` (rad).
    pub fn new(bank_angle: R) -> Self {
        Self {
            bank_angle,
            peak_heat_flux: R::zero(),
            thermal_load: R::zero(),
            blackout_dwell: R::zero(),
        }
    }

    /// Fold one predicted step: the instantaneous wall heat flux, whether comms are denied this
    /// step, and the step size `dt`.
    pub fn observe(&mut self, heat_flux: R, gnss_denied: bool, dt: R) {
        if heat_flux > self.peak_heat_flux {
            self.peak_heat_flux = heat_flux;
        }
        self.thermal_load += heat_flux * dt;
        if gnss_denied {
            self.blackout_dwell += dt;
        }
    }

    /// Close the branch with the terminal miss distance, yielding the comparable [`BranchOutcome`].
    pub fn finish(self, miss_distance: R) -> BranchOutcome<R> {
        BranchOutcome {
            bank_angle: self.bank_angle,
            peak_heat_flux: self.peak_heat_flux,
            thermal_load: self.thermal_load,
            miss_distance,
            blackout_dwell: self.blackout_dwell,
        }
    }

    /// Close the branch with a **trajectory-derived** miss: the Euclidean distance from the
    /// branch's terminal position (the report's `"final_truth_state"` leading triple) to the
    /// configured aim point. This replaces modeled miss laws once the branch actually flies —
    /// distinct banks steer distinct terminal states, so their misses separate by dynamics.
    pub fn finish_at(self, terminal_position: [R; 3], aim_point: [R; 3]) -> BranchOutcome<R> {
        let d: [R; 3] = core::array::from_fn(|i| terminal_position[i] - aim_point[i]);
        self.finish(norm3(d))
    }
}

// ---------------------------------------------------------------------------
// [5b] 3-DOF bank-steered lift
// ---------------------------------------------------------------------------

/// The 3-DOF bank-steered ④ aero producer: point-mass drag **and lift**, so the clamped guidance
/// command actually steers the trajectory instead of only reshaping the carrier world.
///
/// Each step it forms the peak dynamic pressure `q = ½·ρ_ref·U_max²` from the marcher's `"speed"`
/// field (override with [`with_speed_field`](Self::with_speed_field)), takes the drag acceleration
/// `D = (C_d·A/m)·q` **anti-parallel to the truth velocity**, and adds the lift `L = (L/D)·D`
/// rotated about the velocity vector by the bank angle read from the field's control channel —
/// the value [`CyberneticCorrect`] clamped at the **previous** step, so the actuation carries a
/// one-step lag by construction (command at step `k` flies at step `k+1`). The lift-plane basis
/// comes from the local radial at the truth position: zero bank puts the lift in the
/// radial-velocity plane (pure in-plane lift-up); positive bank rotates it toward `v̂ × n̂`.
/// The full 3-vector lands in the aero-force channel the trajectory kick reads.
///
/// Degenerate geometry falls back conservatively: without a `"speed"` field the stage writes a
/// **zero** aero force (no dynamic pressure this step — an earlier step's force must not latch);
/// without a 6-cell `"truth_state"` it writes the axis-aligned drag `[−D, 0, 0]` (the
/// [`AeroForceCoupling`](super::AeroForceCoupling) behavior); with a vanishing velocity or a
/// velocity parallel to the radial (no lift plane) it writes pure drag. This is deliberately
/// 3-DOF: attitude dynamics, trim, and control surfaces (6-DOF) are out of scope — there is no
/// flight-data anchor to validate them against.
#[derive(Debug, Clone, Copy)]
pub struct BankSteeredLift<R: CfdScalar> {
    speed_field: &'static str,
    truth_field: &'static str,
    rho_ref: R,
    cd_area_over_mass: R,
    lift_over_drag: R,
}

impl<R: CfdScalar> BankSteeredLift<R> {
    /// A 3-DOF aero producer with freestream reference density `rho_ref`, ballistic-coefficient
    /// bundle `cd_area_over_mass = C_d·A/m`, and lift-to-drag ratio `lift_over_drag`.
    pub fn new(rho_ref: R, cd_area_over_mass: R, lift_over_drag: R) -> Self {
        Self {
            speed_field: "speed",
            truth_field: "truth_state",
            rho_ref,
            cd_area_over_mass,
            lift_over_drag,
        }
    }

    /// Form the dynamic pressure from a different speed field, e.g. the compressible carrier's
    /// single-cell `"flight_speed"` when the trajectory should feel the freestream rather than
    /// the post-shock layer.
    pub fn with_speed_field(mut self, field: &'static str) -> Self {
        self.speed_field = field;
        self
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for BankSteeredLift<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(speed) = field.scalar(self.speed_field) else {
            // No dynamic pressure this step: zero the ④ channel, so an aero force written on an
            // earlier step does not stay latched and keep kicking the trajectory.
            field.set_aero_force([R::zero(); 3]);
            return Ok(());
        };
        let u_max = peak(speed);
        let half = R::from_f64(0.5).unwrap_or_else(R::one);
        let q = half * self.rho_ref * u_max * u_max;
        let a_drag = self.cd_area_over_mass * q;

        let truth = field.scalar(self.truth_field);
        let Some([rx, ry, rz, vx, vy, vz]) = (match truth {
            Some([a, b, c, d, e, f]) => Some([*a, *b, *c, *d, *e, *f]),
            _ => None,
        }) else {
            field.set_aero_force([R::zero() - a_drag, R::zero(), R::zero()]);
            return Ok(());
        };

        let eps = R::from_f64(1.0e-12).unwrap_or_else(R::zero);
        let v = [vx, vy, vz];
        let v_norm = norm3(v);
        if v_norm <= eps {
            field.set_aero_force([R::zero() - a_drag, R::zero(), R::zero()]);
            return Ok(());
        }
        let v_hat = scale3(v, R::one() / v_norm);
        let drag = scale3(v_hat, R::zero() - a_drag);

        // The lift plane: the local radial at the truth position, projected off the velocity.
        let r = [rx, ry, rz];
        let r_norm = norm3(r);
        let n_raw = if r_norm > eps {
            let r_hat = scale3(r, R::one() / r_norm);
            let along = dot3(r_hat, v_hat);
            [
                r_hat[0] - along * v_hat[0],
                r_hat[1] - along * v_hat[1],
                r_hat[2] - along * v_hat[2],
            ]
        } else {
            [R::zero(); 3]
        };
        let n_norm = norm3(n_raw);
        if n_norm <= eps {
            // Velocity along the radial: no lift plane, pure drag.
            field.set_aero_force(drag);
            return Ok(());
        }
        let n_hat = scale3(n_raw, R::one() / n_norm);
        let b_hat = cross3(v_hat, n_hat);

        // The clamped bank from the control channel (the previous step's gate output).
        let bank = field.control_action().unwrap_or_else(R::zero);
        let a_lift = self.lift_over_drag * a_drag;
        let (sin_b, cos_b) = (bank.sin(), bank.cos());
        let aero: [R; 3] =
            core::array::from_fn(|i| drag[i] + a_lift * (cos_b * n_hat[i] + sin_b * b_hat[i]));
        field.set_aero_force(aero);
        Ok(())
    }
}
