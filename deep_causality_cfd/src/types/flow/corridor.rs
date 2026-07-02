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
use crate::types::CfdScalar;
use crate::types::flow::BlackoutTrigger;
use alloc::format;
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

/// The classifier's decision at a step: the selected [`GoverningModel`], the Knudsen number it was
/// selected from, and the plasma/comms state (angular plasma frequency + whether GNSS is denied).
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
}

impl<R: CfdScalar> RegimeClass<R> {
    /// The governing-model + comms-denial pair that identifies a *regime* for change detection
    /// (the continuous Knudsen/plasma-frequency values are excluded — only a band or denial
    /// transition is a regime change worth logging).
    fn key(&self) -> (GoverningModel, bool) {
        (self.model, self.gnss_denied)
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
        }
    }

    /// Override the Knudsen band thresholds (`slip ≤ transitional ≤ free_molecular`).
    pub fn with_thresholds(mut self, slip: R, transitional: R, free_molecular: R) -> Self {
        self.slip_threshold = slip;
        self.transitional_threshold = transitional;
        self.free_molecular_threshold = free_molecular;
        self
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
        };

        // Log only genuine regime transitions (model band or comms-denial change).
        let changed = field.regime().map(|prev| prev.key()) != Some(class.key());
        if changed {
            let denial = if class.gnss_denied {
                "GNSS-denied"
            } else {
                "GNSS-available"
            };
            field.log_mut().add_entry(&format!(
                "regime -> {} ({}), Kn={}",
                model.name(),
                denial,
                kn,
            ));
        }
        field.set_regime(class);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// [4] Trajectory / navigation stage
// ---------------------------------------------------------------------------

/// The trajectory/navigation stage ([4]): one [`ReentryNavEngine`](crate::ReentryNavEngine) step per coupling step — KS
/// predict with the ④ aero-force channel as the perturbation kick, then the ESKF measurement fold.
///
/// The nav *state* threads through the [`CoupledField`] (the stage takes the engine out, advances
/// it, and puts it back — stages stay immutable, so the same stage value drives every step and a
/// forked field carries its own engine). A no-op if no engine has been seeded with
/// [`CoupledField::set_nav`].
///
/// Measurements are consumed from named scalar fields a sensor stage publishes:
/// * `"gnss_fix"` (3 cells, a measured Cartesian position) — folded **only when GNSS is available**;
///   the gate is the classifier's [`RegimeClass::gnss_denied`] flag on the field (no classifier ⇒
///   available). This is the ④ blackout gating of the corridor.
/// * `"optical_fix"` (3 cells) — the through-plasma optical fix, folded regardless of denial.
///
/// Each step it publishes `"nav_position"` (3 cells) and `"nav_position_variance"` (1 cell) — the
/// dead-reckoning drift / reacquisition-collapse witnesses — and logs the transition between aided
/// and dead-reckoning navigation to the provenance log (transitions only, not every step).
#[derive(Debug, Clone, Copy)]
pub struct TrajectoryNav<R: CfdScalar> {
    process_noise: [R; 17],
    gnss_variance: R,
    optical_variance: R,
}

impl<R: CfdScalar> TrajectoryNav<R> {
    /// A trajectory stage with the ESKF `Q` diagonal `process_noise` and the per-axis measurement
    /// variances of the GNSS and through-plasma optical fixes.
    pub fn new(process_noise: [R; 17], gnss_variance: R, optical_variance: R) -> Self {
        Self {
            process_noise,
            gnss_variance,
            optical_variance,
        }
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
        // Predict: KS drift + the ④ aero acceleration as the Strang kick (zero if no producer ran).
        let aero = field.aero_force().unwrap_or([R::zero(); 3]);
        if let Err(e) = engine.predict(ctx.dt(), aero, self.process_noise) {
            // Thread the engine back before short-circuiting, so the pause/fork state stays whole.
            field.set_nav(engine);
            return Err(e);
        }

        // Correct: GNSS gated by the classifier's denial flag; optical rides through the plasma.
        let denied = field.regime().map(|r| r.gnss_denied).unwrap_or(false);
        let mut aided = false;
        if !denied && let Some(fix) = fix3(field.scalar("gnss_fix")) {
            engine.correct_position(fix, self.gnss_variance);
            aided = true;
        }
        if let Some(fix) = fix3(field.scalar("optical_fix")) {
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
}

// ---------------------------------------------------------------------------
// [6] Cybernetic bounded-correction gate
// ---------------------------------------------------------------------------

/// The verified safety envelope — the cybernetic loop's Context `C`. A bank-angle correction is
/// admissible only inside it; the gate clamps into `[−max_bank, max_bank]` and refuses (yields
/// [`BankCorrection::NoSafeAction`]) once the sensed heat flux or g-load exceeds its ceiling.
#[derive(Debug, Clone, Copy)]
pub struct SafetyEnvelope<R: CfdScalar> {
    /// Maximum admissible wall heat flux (W·m⁻²).
    pub max_heat_flux: R,
    /// Maximum admissible g-load (multiples of g).
    pub max_g_load: R,
    /// Maximum admissible bank-angle magnitude (rad).
    pub max_bank_rad: R,
}

impl<R: CfdScalar> SafetyEnvelope<R> {
    /// An envelope bounded by a heat-flux ceiling, a g-load ceiling, and a bank-angle magnitude.
    pub fn new(max_heat_flux: R, max_g_load: R, max_bank_rad: R) -> Self {
        Self {
            max_heat_flux,
            max_g_load,
            max_bank_rad,
        }
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
    envelope: SafetyEnvelope<R>,
}

impl<R: CfdScalar> CyberneticCorrect<R> {
    /// A gate enforcing `envelope`, reading `"heat_flux"` and `"g_load"` from the coupled field.
    pub fn new(envelope: SafetyEnvelope<R>) -> Self {
        Self {
            heat_flux_field: "heat_flux",
            g_load_field: "g_load",
            envelope,
        }
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
                Ok(())
            }
            BankCorrection::NoSafeAction => {
                field
                    .log_mut()
                    .add_entry("safety-envelope breach: no recoverable bank correction");
                Err(PhysicsError::PhysicalInvariantBroken(
                    "cybernetic gate: safety envelope breached, no recoverable bank correction"
                        .into(),
                ))
            }
        }
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
