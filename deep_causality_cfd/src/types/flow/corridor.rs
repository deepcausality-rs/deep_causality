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
