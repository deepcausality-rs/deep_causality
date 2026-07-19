/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage 3 — corridor **composition** stages that fill the Stage-0 ④ seam (design §"Stage 3"):
//!
//! * [`RegimeClassify`] — the governing-model selector ([2]/[3]): reads the flow's rarefaction
//!   (Knudsen number, from a `"mean_free_path"` field and a configured characteristic length) and the
//!   plasma state (`"n_e"` → [`BlackoutTrigger`] → GNSS-denied), selects the [`GoverningModel`], and
//!   **logs every regime change** into the [`CoupledField`] provenance log.
//! * [`TrajectoryNav`] — the trajectory/navigation stage ([4]): one [`ReentryNavEngine`](crate::ReentryNavEngine) step per
//!   coupling step (KS predict with the ④ aero force as the kick, ESKF correct with GNSS gated by
//!   the classifier's denial flag, through-plasma optical ungated); the nav *state* threads through
//!   the [`CoupledField`], so the stage itself stays immutable.
//! * [`BranchOutcome`] / [`BranchAccumulator`] — the counterfactual bank-angle branch vocabulary
//!   ([5]): a predict-only rollout folds per-step `(heat flux, comms-denied, dt)` samples into the
//!   `(peak heat, thermal load, blackout dwell)` triple and closes with the terminal miss distance.
//!   The rollout *driver* (the alternate-world `run_coupled`) lands with the DSL in Stage 4; this is
//!   the outcome type and its tested reducer the driver feeds.
//! * [`CyberneticCorrect`] — the **bounded-correction gate** ([6]): a [`PhysicsStage`] wrapping a
//!   direct [`CyberneticLoop::control_step`] (not the Effect monad — the committed corrective inner
//!   loop is latency-bound). The loop's Context `C` *is* the verified [`SafetyEnvelope`]; `decide`
//!   clamps the bank-angle Action into it by construction and yields [`BankCorrection::NoSafeAction`]
//!   when no safe action exists, which the gate turns into an `Err` that short-circuits the coupling.
//!   Deterministic (identical inputs → identical action), no Effect-monad allocation on the hot path.

use super::coupling::{CoupledField, PhysicsStage, StepContext};
use crate::CfdScalar;
use crate::navigation::ImuModel;
use crate::types::flow::BlackoutTrigger;
use alloc::format;
use alloc::string::String;
use deep_causality_haft::{CyberneticLoop, HKT5Unbound, LogAddEntry, NoConstraint, Satisfies};
use deep_causality_physics::{ElectronDensity, Length, PhysicsError, knudsen_number_kernel};

// ---------------------------------------------------------------------------
// [2]/[3] Regime classification
// ---------------------------------------------------------------------------

/// The governing continuum/rarefaction model selected from the Knudsen number. The classic bands:
/// continuum Navier–Stokes below `Kn ≈ 0.01`, slip-corrected continuum to `≈ 0.1`, transitional to
/// `≈ 10`, free-molecular above. (Thresholds are configurable on [`RegimeClassify`].)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoverningModel {
    /// Continuum Navier–Stokes (`Kn < slip_threshold`).
    Continuum,
    /// Slip-corrected continuum (`slip_threshold ≤ Kn < transitional_threshold`).
    Slip,
    /// Transitional regime (`transitional_threshold ≤ Kn < free_molecular_threshold`).
    Transitional,
    /// Free-molecular flow (`Kn ≥ free_molecular_threshold`).
    FreeMolecular,
}

impl GoverningModel {
    /// A short, stable label for provenance messages.
    pub fn name(self) -> &'static str {
        match self {
            GoverningModel::Continuum => "continuum",
            GoverningModel::Slip => "slip",
            GoverningModel::Transitional => "transitional",
            GoverningModel::FreeMolecular => "free-molecular",
        }
    }
}

/// The compressibility band of the flight phase (change `add-retropulsion-coupled-stages`,
/// capability `flight-regime-classifier`), read from the carrier-published `"flight_mach"`.
/// [`Unknown`](Self::Unknown) is the neutral value a world that publishes no Mach carries, so the
/// corridor's classification is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachRegime {
    /// No `"flight_mach"` published — the axis is neutral.
    Unknown,
    /// `M ≤ subsonic_ceiling`.
    Subsonic,
    /// Between the subsonic ceiling and the supersonic floor.
    Transonic,
    /// `M ≥ supersonic_floor`.
    Supersonic,
}

impl MachRegime {
    /// A short, stable label for provenance messages.
    pub fn name(self) -> &'static str {
        match self {
            MachRegime::Unknown => "mach-unknown",
            MachRegime::Subsonic => "subsonic",
            MachRegime::Transonic => "transonic",
            MachRegime::Supersonic => "supersonic",
        }
    }
}

/// The propulsion state of the flight phase, read from the `"ignited"` flag.
/// [`Unknown`](Self::Unknown) is the neutral value for a world that carries no propulsion state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrustState {
    /// No `"ignited"` scalar published — the axis is neutral.
    Unknown,
    /// Carried but unlit.
    Coast,
    /// Lit.
    Burn,
}

impl ThrustState {
    /// A short, stable label for provenance messages.
    pub fn name(self) -> &'static str {
        match self {
            ThrustState::Unknown => "thrust-unknown",
            ThrustState::Coast => "coast",
            ThrustState::Burn => "burn",
        }
    }
}

/// The classifier's decision at a step: the selected [`GoverningModel`], the Knudsen number it was
/// selected from, the plasma/comms state (angular plasma frequency + whether GNSS is denied), and
/// the powered-descent flight phase (Mach band, thrust state, touchdown).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegimeClass<R: CfdScalar> {
    /// The selected governing model.
    pub model: GoverningModel,
    /// The Knudsen number `Kn = λ / L` the model was selected from.
    pub knudsen: R,
    /// The angular plasma frequency `ω_p` (rad/s) at the peak electron density.
    pub plasma_frequency: R,
    /// Whether GNSS / comms are denied (plasma frequency above the configured band).
    pub gnss_denied: bool,
    /// The compressibility band (neutral when no `"flight_mach"` is published).
    pub mach_regime: MachRegime,
    /// The propulsion state (neutral when no `"ignited"` flag is published).
    pub thrust_state: ThrustState,
    /// Whether the vehicle is at or below the configured touchdown altitude floor.
    pub touchdown: bool,
}

impl<R: CfdScalar> RegimeClass<R> {
    /// The discrete tuple that identifies a *regime* for change detection: the governing model,
    /// comms denial, and the three flight-phase axes. The continuous Knudsen / plasma-frequency /
    /// Mach values are excluded — only a band, denial, thrust, or touchdown transition is a regime
    /// change worth logging. For a world that publishes none of the flight scalars the three new
    /// components are constant, so change detection reduces to today's `(model, gnss_denied)` pair
    /// and the corridor's logged transitions are unchanged.
    fn key(&self) -> (GoverningModel, bool, MachRegime, ThrustState, bool) {
        (
            self.model,
            self.gnss_denied,
            self.mach_regime,
            self.thrust_state,
            self.touchdown,
        )
    }
}

/// The governing-model selector (\[2\]/\[3\]). Reads the peak mean free path from a `"mean_free_path"`
/// field and forms `Kn = λ / L` against the configured characteristic length, reads the peak
/// electron density from `"n_e"` and maps it through a [`BlackoutTrigger`] to the GNSS-denial flag,
/// then records the [`RegimeClass`] on the field — logging a provenance entry whenever the regime
/// (governing model or comms-denial) changes. A no-op if `"mean_free_path"` is absent.
#[derive(Debug, Clone, Copy)]
pub struct RegimeClassify<R: CfdScalar> {
    mfp_field: &'static str,
    ne_field: &'static str,
    characteristic_length: R,
    slip_threshold: R,
    transitional_threshold: R,
    free_molecular_threshold: R,
    trigger: BlackoutTrigger<R>,
    mach_field: &'static str,
    ignited_field: &'static str,
    altitude_field: &'static str,
    /// The powered-descent bands, **opt-in**. `None` — the default — leaves all three flight axes
    /// neutral *even when the carrier publishes their scalars*, so the corridor (whose carrier
    /// publishes `"flight_mach"` every step) classifies and logs exactly as before.
    flight_axes: Option<FlightAxes<R>>,
}

/// The band edges the powered-descent flight axes are read against (see
/// [`RegimeClassify::with_flight_axes`]).
#[derive(Debug, Clone, Copy)]
struct FlightAxes<R: CfdScalar> {
    subsonic_ceiling: R,
    supersonic_floor: R,
    touchdown_altitude: R,
}

impl<R: CfdScalar> RegimeClassify<R> {
    /// A classifier over the standard Knudsen bands (`0.01` / `0.1` / `10`) for a body of
    /// characteristic length `characteristic_length` (m), with `trigger` mapping the peak electron
    /// density to the GNSS-denial decision.
    pub fn new(characteristic_length: R, trigger: BlackoutTrigger<R>) -> Self {
        Self {
            mfp_field: "mean_free_path",
            ne_field: "n_e",
            characteristic_length,
            slip_threshold: R::from_f64(0.01).unwrap_or_else(R::zero),
            transitional_threshold: R::from_f64(0.1).unwrap_or_else(R::zero),
            free_molecular_threshold: R::from_f64(10.0).unwrap_or_else(R::one),
            trigger,
            mach_field: "flight_mach",
            ignited_field: "ignited",
            altitude_field: "flight_altitude",
            flight_axes: None,
        }
    }

    /// Override the Knudsen band thresholds (`slip ≤ transitional ≤ free_molecular`).
    pub fn with_thresholds(mut self, slip: R, transitional: R, free_molecular: R) -> Self {
        self.slip_threshold = slip;
        self.transitional_threshold = transitional;
        self.free_molecular_threshold = free_molecular;
        self
    }

    /// **Opt into** the powered-descent flight axes: the subsonic ceiling and supersonic floor the
    /// `"flight_mach"` scalar is banded against, and the altitude at or below which the vehicle
    /// counts as touched down.
    ///
    /// Without this call the three axes stay neutral **even though the compressible carrier
    /// publishes `"flight_mach"` every step**, so a corridor world's classification, regime key, and
    /// logged messages are exactly the pre-change ones. Only a burn-phase world opts in.
    pub fn with_flight_axes(
        mut self,
        subsonic_ceiling: R,
        supersonic_floor: R,
        touchdown_altitude: R,
    ) -> Self {
        self.flight_axes = Some(FlightAxes {
            subsonic_ceiling,
            supersonic_floor,
            touchdown_altitude,
        });
        self
    }

    /// Band the published flight Mach; neutral when the world publishes none.
    fn mach_regime_of(&self, field: &CoupledField<R>) -> MachRegime {
        let Some(axes) = self.flight_axes else {
            return MachRegime::Unknown;
        };
        let Some(mach) = field
            .scalar(self.mach_field)
            .and_then(|s| s.first().copied())
        else {
            return MachRegime::Unknown;
        };
        if mach <= axes.subsonic_ceiling {
            MachRegime::Subsonic
        } else if mach >= axes.supersonic_floor {
            MachRegime::Supersonic
        } else {
            MachRegime::Transonic
        }
    }

    /// Read the propulsion state from the `"ignited"` flag; neutral when the world carries none.
    fn thrust_state_of(&self, field: &CoupledField<R>) -> ThrustState {
        if self.flight_axes.is_none() {
            return ThrustState::Unknown;
        }
        match field
            .scalar(self.ignited_field)
            .and_then(|s| s.first().copied())
        {
            None => ThrustState::Unknown,
            Some(f) if f > R::zero() => ThrustState::Burn,
            Some(_) => ThrustState::Coast,
        }
    }

    /// Whether the published altitude is at or below the touchdown floor (false when unpublished).
    fn touchdown_of(&self, field: &CoupledField<R>) -> bool {
        let Some(axes) = self.flight_axes else {
            return false;
        };
        field
            .scalar(self.altitude_field)
            .and_then(|s| s.first().copied())
            .is_some_and(|alt| alt <= axes.touchdown_altitude)
    }

    /// Select the governing model for a Knudsen number against the configured bands.
    fn model_for(&self, kn: R) -> GoverningModel {
        if kn < self.slip_threshold {
            GoverningModel::Continuum
        } else if kn < self.transitional_threshold {
            GoverningModel::Slip
        } else if kn < self.free_molecular_threshold {
            GoverningModel::Transitional
        } else {
            GoverningModel::FreeMolecular
        }
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for RegimeClassify<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(mfp) = field.scalar(self.mfp_field) else {
            return Ok(());
        };
        // Rarefaction is worst where the mean free path is largest, so classify off its peak.
        let mfp_peak = peak(mfp);
        let length = Length::new(self.characteristic_length)?;
        let kn = knudsen_number_kernel(mfp_peak, &length)?;
        let model = self.model_for(kn);

        // The comms side: peak electron density → plasma frequency → GNSS-denial decision.
        let ne_peak = field
            .scalar(self.ne_field)
            .map(peak)
            .unwrap_or_else(R::zero);
        let blackout = self.trigger.evaluate(ElectronDensity::new(ne_peak)?)?;

        let class = RegimeClass {
            model,
            knudsen: kn,
            plasma_frequency: blackout.plasma_frequency,
            gnss_denied: blackout.denied,
            mach_regime: self.mach_regime_of(field),
            thrust_state: self.thrust_state_of(field),
            touchdown: self.touchdown_of(field),
        };

        // Log only genuine regime transitions (model band, comms-denial, or a flight-phase change).
        let changed = field.regime().map(|prev| prev.key()) != Some(class.key());
        if changed {
            let denial = if class.gnss_denied {
                "GNSS-denied"
            } else {
                "GNSS-available"
            };
            // The flight-phase suffix appears only when a powered-descent axis is live, so a world
            // publishing none of the flight scalars logs exactly the pre-change message.
            let phase = if class.mach_regime == MachRegime::Unknown
                && class.thrust_state == ThrustState::Unknown
                && !class.touchdown
            {
                String::new()
            } else {
                format!(
                    ", {} / {}{}",
                    class.mach_regime.name(),
                    class.thrust_state.name(),
                    if class.touchdown { ", touchdown" } else { "" },
                )
            };
            field.log_mut().add_entry(&format!(
                "regime -> {} ({}), Kn={}{}",
                model.name(),
                denial,
                kn,
                phase,
            ));
        }
        field.set_regime(class);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// [4] Trajectory / navigation stage
// ---------------------------------------------------------------------------

/// The trajectory/navigation stage (\[4\]): one [`ReentryNavEngine`](crate::ReentryNavEngine) step per coupling step — KS
/// predict with the ④ aero-force channel as the perturbation kick, then the ESKF measurement fold.
///
/// The nav *state* threads through the [`CoupledField`] (the stage takes the engine out, advances
/// it, and puts it back — stages stay immutable, so the same stage value drives every step and a
/// forked field carries its own engine). A no-op if no engine has been seeded with
/// [`CoupledField::set_nav`].
///
/// Measurements are **consumed** (taken out of the field, one-shot) from named scalar fields a
/// sensor stage publishes each step — a publisher that goes quiet leaves no stale fix behind to
/// be re-folded as fresh:
/// * `"gnss_fix"` (3 cells, a measured Cartesian position) — folded **only when GNSS is available**;
///   the gate is the classifier's [`RegimeClass::gnss_denied`] flag on the field (no classifier ⇒
///   available). This is the ④ blackout gating of the corridor. A denied-step fix is consumed
///   unread.
/// * `"optical_fix"` (3 cells) — the through-plasma optical fix, folded regardless of denial.
///
/// Each step it publishes `"nav_position"` (3 cells) and `"nav_position_variance"` (1 cell) — the
/// dead-reckoning drift / reacquisition-collapse witnesses — and logs the transition between aided
/// and dead-reckoning navigation to the provenance log (transitions only, not every step).
///
/// With [`with_imu`](Self::with_imu) the predict integrates the **IMU-sensed** specific force
/// (the ④ aero plus the accelerometer bias) instead of the true value — the real INS
/// dead-reckoning error mechanism — and the ESKF `Q` comes from the IMU's grade.
#[derive(Debug, Clone, Copy)]
pub struct TrajectoryNav<R: CfdScalar> {
    process_noise: [R; 17],
    gnss_variance: R,
    optical_variance: R,
    imu: Option<ImuModel<R>>,
}

impl<R: CfdScalar> TrajectoryNav<R> {
    /// A trajectory stage with the ESKF `Q` diagonal `process_noise` and the per-axis measurement
    /// variances of the GNSS and through-plasma optical fixes. The predict integrates the true ④
    /// specific force; use [`with_imu`](Self::with_imu) for a sensed one.
    pub fn new(process_noise: [R; 17], gnss_variance: R, optical_variance: R) -> Self {
        Self {
            process_noise,
            gnss_variance,
            optical_variance,
            imu: None,
        }
    }

    /// Sense the ④ specific force through `imu` before the predict (accelerometer bias included),
    /// and take the ESKF `Q` diagonal from the IMU's grade.
    pub fn with_imu(mut self, imu: ImuModel<R>) -> Self {
        self.process_noise = imu.process_noise();
        self.imu = Some(imu);
        self
    }
}

/// Read a 3-cell fix field, if present and well-formed.
fn fix3<R: CfdScalar>(xs: Option<&[R]>) -> Option<[R; 3]> {
    match xs {
        Some([x, y, z]) => Some([*x, *y, *z]),
        _ => None,
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for TrajectoryNav<R> {
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(mut engine) = field.take_nav() else {
            return Ok(());
        };
        // Predict: KS drift + the ④ aero acceleration as the Strang kick (zero if no producer
        // ran), sensed through the IMU when one is configured.
        let aero = field.aero_force().unwrap_or([R::zero(); 3]);
        let sensed = match &self.imu {
            Some(imu) => imu.sense_specific_force(aero),
            None => aero,
        };
        if let Err(e) = engine.predict(ctx.dt(), sensed, self.process_noise) {
            // Thread the engine back before short-circuiting, so the pause/fork state stays whole.
            field.set_nav(engine);
            return Err(e);
        }

        // Correct: GNSS gated by the classifier's denial flag; optical rides through the plasma.
        // Each fix is a **consumed, one-shot** measurement: taking it out of the field means a
        // publisher that goes quiet leaves nothing behind, so a stale fix is never re-folded as
        // fresh on a later step (which would hold the filter in aided mode and over-collapse the
        // covariance). A denied-step GNSS fix is consumed unread — the broadcast the receiver
        // could not use is gone, not latched for reacquisition.
        let gnss = field.take_scalar("gnss_fix");
        let optical = field.take_scalar("optical_fix");
        let denied = field.regime().map(|r| r.gnss_denied).unwrap_or(false);
        let mut aided = false;
        if !denied && let Some(fix) = fix3(gnss.as_deref()) {
            engine.correct_position(fix, self.gnss_variance);
            aided = true;
        }
        if let Some(fix) = fix3(optical.as_deref()) {
            engine.correct_position(fix, self.optical_variance);
            aided = true;
        }

        // Log aided <-> dead-reckoning transitions only (the mode is carried on the field).
        let mode = if aided { R::one() } else { R::zero() };
        let prev = field.scalar("nav_mode").and_then(|m| m.first().copied());
        if prev != Some(mode) {
            let label = if aided {
                "nav: aided (position fix folded)"
            } else {
                "nav: dead reckoning (no usable fix)"
            };
            field.log_mut().add_entry(label);
        }
        field.set_scalar("nav_mode", Vec::from([mode]));

        // Publish the drift / reacquisition witnesses and thread the engine back.
        field.set_scalar("nav_position", engine.position().to_vec());
        field.set_scalar(
            "nav_position_variance",
            Vec::from([engine.position_variance()]),
        );
        field.set_nav(engine);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// [5] Counterfactual bank-angle branch outcome
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// [6] Cybernetic bounded-correction gate
// ---------------------------------------------------------------------------

/// The optional powered-descent axes of a [`SafetyEnvelope`] (change
/// `plasma-retropulsion-cfd-contracts`, capability `powered-descent-envelope`). Present only for a
/// burn-phase world; absent (`SafetyEnvelope::burn == None`) for the corridor, where the gate
/// behaves exactly as before. Carries the throttle floor/ceiling, the maximum thrust coefficient
/// `max_ct` (the *dynamic* throttle cap — the admissible ceiling is the static ceiling min'd with
/// the throttle at which `C_T = T/(q∞·S_ref)` reaches `max_ct`), the ignition dynamic-pressure
/// window `[q_min, q_max]` (stored for M4's ignition-corridor commit, not enforced by the gate),
/// the propellant floor, and the maximum descent rate.
#[derive(Debug, Clone, Copy)]
pub struct BurnEnvelope<R: CfdScalar> {
    /// Minimum admissible throttle command.
    pub throttle_floor: R,
    /// Maximum admissible throttle command (the static ceiling).
    pub throttle_ceiling: R,
    /// Maximum admissible thrust coefficient `C_T` (the dynamic, q-dependent throttle cap).
    pub max_ct: R,
    /// Ignition dynamic-pressure window lower bound (Pa) — stored for M4, not gated here.
    pub q_min: R,
    /// Ignition dynamic-pressure window upper bound (Pa) — stored for M4, not gated here.
    pub q_max: R,
    /// Minimum admissible propellant mass (kg); a positive throttle at or below it is a breach.
    pub propellant_floor: R,
    /// Maximum admissible descent rate (m/s).
    pub max_descent_rate: R,
}

impl<R: CfdScalar> BurnEnvelope<R> {
    /// A burn envelope with all six powered-descent axes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        throttle_floor: R,
        throttle_ceiling: R,
        max_ct: R,
        q_min: R,
        q_max: R,
        propellant_floor: R,
        max_descent_rate: R,
    ) -> Self {
        Self {
            throttle_floor,
            throttle_ceiling,
            max_ct,
            q_min,
            q_max,
            propellant_floor,
            max_descent_rate,
        }
    }
}

/// The verified safety envelope — the cybernetic loop's Context `C`. A bank-angle correction is
/// admissible only inside it; the gate clamps into `[−max_bank, max_bank]` and refuses (yields
/// [`BankCorrection::NoSafeAction`]) once the sensed heat flux or g-load exceeds its ceiling.
///
/// The optional [`burn`](Self::burn) axes carry the powered-descent limits; they are `None` for the
/// corridor, so the gate stays bit-identical until a burn-phase world attaches them.
#[derive(Debug, Clone, Copy)]
pub struct SafetyEnvelope<R: CfdScalar> {
    /// Maximum admissible wall heat flux (W·m⁻²).
    pub max_heat_flux: R,
    /// Maximum admissible g-load (multiples of g).
    pub max_g_load: R,
    /// Maximum admissible bank-angle magnitude (rad).
    pub max_bank_rad: R,
    /// The optional powered-descent axes; `None` for the corridor (gate unchanged).
    pub burn: Option<BurnEnvelope<R>>,
}

impl<R: CfdScalar> SafetyEnvelope<R> {
    /// An envelope bounded by a heat-flux ceiling, a g-load ceiling, and a bank-angle magnitude.
    /// The powered-descent axes are absent (`burn: None`); attach them with [`with_burn`](Self::with_burn).
    pub fn new(max_heat_flux: R, max_g_load: R, max_bank_rad: R) -> Self {
        Self {
            max_heat_flux,
            max_g_load,
            max_bank_rad,
            burn: None,
        }
    }

    /// Attach the powered-descent axes, returning the extended envelope (the corridor never calls
    /// this, so its gate is untouched).
    pub fn with_burn(mut self, burn: BurnEnvelope<R>) -> Self {
        self.burn = Some(burn);
        self
    }
}

/// The sensed coupled state the gate observes — the loop's Sensor `S`.
#[derive(Debug, Clone, Copy)]
struct SensedState<R: CfdScalar> {
    peak_heat_flux: R,
    g_load: R,
}

/// The estimated margins — the loop's Belief `B` (headroom to each envelope ceiling).
#[derive(Debug, Clone, Copy)]
struct MarginBelief<R: CfdScalar> {
    thermal_margin: R,
    g_margin: R,
}

/// The bounded bank-angle correction — the loop's Action `A`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BankCorrection<R: CfdScalar> {
    /// A bank-angle command clamped into the envelope (rad).
    Clamped(R),
    /// No admissible correction exists — the envelope is already breached (the loop's Entropy `E`).
    NoSafeAction,
}

/// The five-slot agent carrying `(S, B, C, A, E)` for the [`CyberneticLoop`] witness.
struct GuidanceAgent<S, B, C, A, E>(S, B, C, A, E);

/// The HKT-5 witness realizing the corridor's cybernetic guidance loop.
struct GuidanceWitness;

impl HKT5Unbound for GuidanceWitness {
    type Constraint = NoConstraint;
    type Type<S, B, C, A, E> = GuidanceAgent<S, B, C, A, E>;
}

impl CyberneticLoop<GuidanceWitness> for GuidanceWitness {
    fn control_step<S, B, C, A, E, FObserve, FDecide>(
        agent: GuidanceAgent<S, B, C, A, E>,
        sensor_input: S,
        observe_fn: FObserve,
        decide_fn: FDecide,
    ) -> Result<A, E>
    where
        S: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
        E: Satisfies<NoConstraint>,
        FObserve: Fn(S, &C) -> B,
        FDecide: Fn(B, &C) -> A,
    {
        // The agent carries the single fixed Context (its 3rd slot); observe and decide both borrow
        // it. The Action encodes the refusal (NoSafeAction), so this witness is infallible in `E`.
        let GuidanceAgent(_s, _b, context, _a, _e) = agent;
        let belief = observe_fn(sensor_input, &context);
        Ok(decide_fn(belief, &context))
    }
}

/// The bounded-correction gate (\[6\]) as a between-step [`PhysicsStage`]. It senses the coupled
/// state (peak `"heat_flux"`, `"g_load"`), runs one [`CyberneticLoop::control_step`] against the
/// [`SafetyEnvelope`], and either writes the clamped bank angle into the field's control channel or —
/// on an unrecoverable breach — logs it and returns `Err`, short-circuiting the coupling (the design's
/// "return Entropy `E`, emit no unsafe action"). The desired bank is the field's current control
/// action (a prior guidance stage's raw command) or `0` if none; the gate only ever *bounds* it.
#[derive(Debug, Clone, Copy)]
pub struct CyberneticCorrect<R: CfdScalar> {
    heat_flux_field: &'static str,
    g_load_field: &'static str,
    q_field: &'static str,
    propellant_field: &'static str,
    descent_rate_field: &'static str,
    thrust_ref: R,
    s_ref: R,
    envelope: SafetyEnvelope<R>,
}

impl<R: CfdScalar> CyberneticCorrect<R> {
    /// A gate enforcing `envelope`, reading `"heat_flux"` and `"g_load"` from the coupled field.
    /// The powered-descent sensing carries inert defaults; enforcement of the burn axes runs only
    /// when the envelope's `burn` axes and a throttle command are both present (see
    /// [`with_burn_sensing`](Self::with_burn_sensing)).
    pub fn new(envelope: SafetyEnvelope<R>) -> Self {
        Self {
            heat_flux_field: "heat_flux",
            g_load_field: "g_load",
            q_field: "q_inf",
            propellant_field: "propellant",
            descent_rate_field: "descent_rate",
            thrust_ref: R::zero(),
            s_ref: R::zero(),
            envelope,
        }
    }

    /// Configure the powered-descent sensing: the scalar-field names carrying the freestream
    /// dynamic pressure, the carried propellant, and the descent rate, plus the full-throttle
    /// thrust reference and aerodynamic reference area the dynamic `C_T` cap is computed from
    /// (`C_T = throttle·thrust_ref / (q∞·s_ref)`). Only read when the envelope's burn axes are
    /// active, so the corridor gate is unaffected.
    pub fn with_burn_sensing(
        mut self,
        q_field: &'static str,
        propellant_field: &'static str,
        descent_rate_field: &'static str,
        thrust_ref: R,
        s_ref: R,
    ) -> Self {
        self.q_field = q_field;
        self.propellant_field = propellant_field;
        self.descent_rate_field = descent_rate_field;
        self.thrust_ref = thrust_ref;
        self.s_ref = s_ref;
        self
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for CyberneticCorrect<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let peak_heat_flux = field
            .scalar(self.heat_flux_field)
            .map(peak)
            .unwrap_or_else(R::zero);
        let g_load = field
            .scalar(self.g_load_field)
            .map(peak)
            .unwrap_or_else(R::zero);
        let desired_bank = field.control_action().unwrap_or_else(R::zero);

        let sensed = SensedState {
            peak_heat_flux,
            g_load,
        };
        let agent = GuidanceAgent(
            sensed,
            MarginBelief {
                thermal_margin: R::zero(),
                g_margin: R::zero(),
            },
            self.envelope,
            BankCorrection::NoSafeAction,
            (),
        );

        // observe: S × &C → B (headroom to each ceiling); decide: B × &C → A (clamp, or refuse).
        let action = GuidanceWitness::control_step(
            agent,
            sensed,
            |s: SensedState<R>, c: &SafetyEnvelope<R>| MarginBelief {
                thermal_margin: c.max_heat_flux - s.peak_heat_flux,
                g_margin: c.max_g_load - s.g_load,
            },
            |b: MarginBelief<R>, c: &SafetyEnvelope<R>| {
                if b.thermal_margin < R::zero() || b.g_margin < R::zero() {
                    BankCorrection::NoSafeAction
                } else {
                    BankCorrection::Clamped(clamp(
                        desired_bank,
                        R::zero() - c.max_bank_rad,
                        c.max_bank_rad,
                    ))
                }
            },
        )
        .unwrap_or(BankCorrection::NoSafeAction);

        match action {
            BankCorrection::Clamped(theta) => {
                if theta != desired_bank {
                    field.log_mut().add_entry(&format!(
                        "bank correction bounded to envelope: {} -> {} rad",
                        desired_bank, theta,
                    ));
                }
                field.set_control_action(theta);
            }
            BankCorrection::NoSafeAction => {
                field
                    .log_mut()
                    .add_entry("safety-envelope breach: no recoverable bank correction");
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "cybernetic gate: safety envelope breached, no recoverable bank correction"
                        .into(),
                ));
            }
        }

        // ── Powered-descent enforcement — inert unless the envelope's burn axes AND a throttle
        //    command are both present, so the corridor gate is bit-identical without them. ──
        if let Some(burn) = self.envelope.burn
            && let Some(commanded) = field.throttle_action()
        {
            // Propellant floor: a positive throttle at or below the floor is an unrecoverable breach.
            let propellant = field
                .scalar(self.propellant_field)
                .map(peak)
                .unwrap_or_else(R::zero);
            if commanded > R::zero() && propellant <= burn.propellant_floor {
                field.log_mut().add_entry(&format!(
                    "safety-envelope breach: propellant {} at/below floor {} under commanded thrust",
                    propellant, burn.propellant_floor,
                ));
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "cybernetic gate: propellant floor breached under commanded thrust".into(),
                ));
            }
            // Descent rate: above the bound with no admissible throttle correction is a breach.
            let descent_rate = field
                .scalar(self.descent_rate_field)
                .map(peak)
                .unwrap_or_else(R::zero);
            if descent_rate > burn.max_descent_rate {
                field.log_mut().add_entry(&format!(
                    "safety-envelope breach: descent rate {} exceeds bound {}",
                    descent_rate, burn.max_descent_rate,
                ));
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "cybernetic gate: descent-rate bound breached, no recoverable throttle".into(),
                ));
            }
            // Throttle clamp into `[floor, min(ceiling, dynamic C_T cap)]`. The dynamic cap is the
            // throttle at which `C_T = throttle·thrust_ref/(q∞·s_ref)` reaches `max_ct`; it moves
            // with the sensed dynamic pressure. Absent a usable thrust reference or sensed q∞, only
            // the static ceiling binds.
            let q_inf = field.scalar(self.q_field).map(peak).unwrap_or_else(R::zero);
            let ceiling = if self.thrust_ref > R::zero() && q_inf > R::zero() {
                let ct_ceiling = burn.max_ct * q_inf * self.s_ref / self.thrust_ref;
                if ct_ceiling < burn.throttle_ceiling {
                    ct_ceiling
                } else {
                    burn.throttle_ceiling
                }
            } else {
                burn.throttle_ceiling
            };
            let bounded = clamp(commanded, burn.throttle_floor, ceiling);
            if bounded != commanded {
                field.log_mut().add_entry(&format!(
                    "throttle command bounded to envelope: {} -> {}",
                    commanded, bounded,
                ));
            }
            field.set_throttle_action(bounded);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// The peak (maximum) of a scalar field, or `0` for an empty field.
fn peak<R: CfdScalar>(xs: &[R]) -> R {
    xs.iter()
        .copied()
        .fold(R::zero(), |a, x| if x > a { x } else { a })
}

/// The Euclidean norm of a 3-vector.
fn norm3<R: CfdScalar>(x: [R; 3]) -> R {
    dot3(x, x).sqrt()
}

/// The dot product of two 3-vectors.
fn dot3<R: CfdScalar>(a: [R; 3], b: [R; 3]) -> R {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A 3-vector scaled by `s`.
fn scale3<R: CfdScalar>(x: [R; 3], s: R) -> [R; 3] {
    [x[0] * s, x[1] * s, x[2] * s]
}

/// The cross product `a × b`.
fn cross3<R: CfdScalar>(a: [R; 3], b: [R; 3]) -> [R; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Clamp `x` into `[lo, hi]`.
fn clamp<R: CfdScalar>(x: R, lo: R, hi: R) -> R {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}
