/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The bounded-correction gate (\[6\]): sense every axis, decide, log every breach, then act.

use super::super::coupling::{CoupledField, PhysicsStage, StepContext};
use super::envelope::{
    BankCorrection, GuidanceAgent, GuidanceWitness, MarginBelief, SafetyEnvelope, SensedState,
};
use super::{clamp, peak};
use crate::CfdScalar;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use deep_causality_haft::{CyberneticLoop, LogAddEntry};
use deep_causality_physics::PhysicsError;

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
    ignited_field: &'static str,
    thrust_ref: R,
    s_ref: R,
    envelope: SafetyEnvelope<R>,
}

/// The world-published throttle scalar the propulsion stages fall back to. The gate senses the
/// throttle *channel* alone and reads this name only to detect the blind-gate misconfiguration —
/// axes attached, thrust commanded through the scalar, nothing on the channel to enforce against.
const COMMANDED_THROTTLE_FIELD: &str = "commanded_throttle";

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
            ignited_field: "ignited",
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

        // ── Sense and decide every axis before acting on any of them. ──
        //
        // A refusal short-circuits every later stage, so the pre-change ordering — where the bank
        // refusal returned before the burn block was reached — was already safe. What it lost was
        // the *record*: a simultaneous propellant-floor or descent-rate breach went unlogged, so the
        // step that most needs a complete account named only one of its causes. Breaches are
        // therefore collected in a fixed axis order, all logged, and the first returned — a
        // deterministic error string with a complete provenance trail behind it.
        let mut breaches: Vec<(String, String)> = Vec::new();

        if matches!(action, BankCorrection::NoSafeAction) {
            breaches.push((
                "safety-envelope breach: no recoverable bank correction".into(),
                "cybernetic gate: safety envelope breached, no recoverable bank correction".into(),
            ));
        }

        // ── Powered-descent enforcement — inert unless the envelope's burn axes are attached, so
        //    the corridor gate is bit-identical without them. ──
        let mut bounded_throttle = None;
        if let Some(burn) = self.envelope.burn {
            let commanded_channel = field.throttle_action();

            // The gate senses the channel alone, while the propulsion stages honour the channel OR
            // the world-published scalar. A world driving only the scalar therefore runs its full
            // propulsion path — thrust, depletion, the drag decrement, the plume re-imprint — with
            // the gate enforcing nothing. Widening the sensing is not the fix: the clamp writes back
            // into the channel, and a channel written from a scalar source would outrank that
            // world's published constant on every later step, freezing a counterfactual branch at
            // its first clamped value. So the misconfiguration is named where it happens.
            if commanded_channel.is_none() {
                let published = field
                    .scalar(COMMANDED_THROTTLE_FIELD)
                    .and_then(|s| s.first().copied())
                    .unwrap_or_else(R::zero);
                if published > R::zero() {
                    breaches.push((
                        format!(
                            "safety-envelope breach: throttle {} commanded through the published \
                             scalar with no throttle channel — the gate cannot enforce a throttle \
                             it cannot see",
                            published
                        ),
                        "cybernetic gate: burn axes attached but the commanded throttle is not on \
                         the throttle channel"
                            .into(),
                    ));
                }
            }

            if let Some(commanded) = commanded_channel {
                // An unusable burn-sensing configuration. `new` leaves the thrust reference at
                // zero, so attaching burn axes without `with_burn_sensing` disables the dynamic
                // `C_T` cap — and *only* that axis, silently, while every other burn axis keeps
                // enforcing. An axis that reports as enforcing while unable to fire is the failure
                // this change exists to remove, so the misconfiguration is refused rather than
                // carried.
                if self.thrust_ref <= R::zero() || self.s_ref <= R::zero() {
                    breaches.push((
                        format!(
                            "safety-envelope misconfiguration: burn axes attached with thrust reference {} \
                             and reference area {} — the dynamic C_T cap cannot bind",
                            self.thrust_ref, self.s_ref,
                        ),
                        "cybernetic gate: burn axes require a positive thrust reference and \
                         reference area (with_burn_sensing)"
                            .into(),
                    ));
                }

                // Propellant floor: a positive throttle at or below the floor is unrecoverable.
                let propellant = field
                    .scalar(self.propellant_field)
                    .map(peak)
                    .unwrap_or_else(R::zero);
                if commanded > R::zero() && propellant <= burn.propellant_floor {
                    breaches.push((
                        format!(
                            "safety-envelope breach: propellant {} at/below floor {} under commanded thrust",
                            propellant, burn.propellant_floor,
                        ),
                        "cybernetic gate: propellant floor breached under commanded thrust".into(),
                    ));
                }

                // Descent rate: above the bound with no admissible throttle correction.
                let descent_rate = field
                    .scalar(self.descent_rate_field)
                    .map(peak)
                    .unwrap_or_else(R::zero);
                if descent_rate > burn.max_descent_rate {
                    breaches.push((
                        format!(
                            "safety-envelope breach: descent rate {} exceeds bound {}",
                            descent_rate, burn.max_descent_rate,
                        ),
                        "cybernetic gate: descent-rate bound breached, no recoverable throttle"
                            .into(),
                    ));
                }

                // Ignition window: `[q_min, q_max]` bounds *when a burn may start*, not how hard it
                // may push, so it applies to the step that lights the engine and not to a burn
                // already under way. The burn-under-way witness is the `"ignited"` flag, which the
                // thrust stage sets from the previous step's clamped command — so on the ignition
                // step itself the flag is still absent, which is exactly the edge this needs. An
                // absent `q∞` sensor does not trip the window: absent sensors read as zero and stay
                // safe (the producer-side gap is closed by the flight-sensor stage, not by failing
                // closed here).
                let sensed_q = field.scalar(self.q_field).map(peak);
                let under_way = field
                    .scalar(self.ignited_field)
                    .and_then(|s| s.first().copied())
                    .is_some_and(|v| v > R::zero());
                if commanded > R::zero()
                    && !under_way
                    && let Some(q_inf) = sensed_q
                    && (q_inf < burn.q_min || q_inf > burn.q_max)
                {
                    breaches.push((
                        format!(
                            "safety-envelope breach: ignition dynamic pressure {} outside the window [{}, {}]",
                            q_inf, burn.q_min, burn.q_max,
                        ),
                        "cybernetic gate: ignition attempted outside the dynamic-pressure window"
                            .into(),
                    ));
                }

                // Throttle clamp into `[floor, min(ceiling, dynamic C_T cap)]`. The dynamic cap is
                // the throttle at which `C_T = throttle·thrust_ref/(q∞·s_ref)` reaches `max_ct`; it
                // moves with the sensed dynamic pressure. Absent a usable thrust reference or
                // sensed q∞, only the static ceiling binds — and with the default `thrust_ref` of
                // zero that is the *only* axis that silently degrades, which `with_burn_sensing`
                // exists to prevent.
                let q_inf = sensed_q.unwrap_or_else(R::zero);
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

                // A crossed window admits no throttle at all. The clamp helper carries no ordering
                // precondition and tests its lower bound first, so a command between the two bounds
                // would be pushed *up* past the `max_ct` cap while a command at or above the floor
                // would be pushed *down* below the stability floor — two out-of-envelope values
                // chosen by the command rather than by any rule. Neither is admissible, and neither
                // is reordering the helper: that satisfies "never below the floor" while silently
                // violating the cap instead.
                if commanded <= R::zero() {
                    // Engine off is always admissible. The throttle floor is a *stability*
                    // constraint for a running engine — below it the jet-penetration flow is
                    // unsteady — so it bounds how softly a burn may run, not whether one must run at
                    // all. Clamping a commanded shutdown up to the floor would light the engine on
                    // the next step, which is exactly what a guidance stage commanding zero before
                    // its ignition corridor commits is asking not to happen.
                    bounded_throttle = Some(commanded);
                } else if ceiling < burn.throttle_floor {
                    breaches.push((
                        format!(
                            "safety-envelope breach: throttle window crossed — ceiling {} below floor {}, no admissible throttle",
                            ceiling, burn.throttle_floor,
                        ),
                        "cybernetic gate: throttle window crossed, no admissible throttle".into(),
                    ));
                } else {
                    bounded_throttle = Some(clamp(commanded, burn.throttle_floor, ceiling));
                }
            }
        }

        // ── Log every breach, then refuse on the first in axis order. ──
        if !breaches.is_empty() {
            for (message, _) in &breaches {
                field.log_mut().add_entry(message);
            }
            let (_, error) = breaches.swap_remove(0);
            return Err(PhysicsError::PhysicalInvariantBroken(error));
        }

        // ── No breach: apply the clamps. ──
        if let BankCorrection::Clamped(theta) = action {
            if theta != desired_bank {
                field.log_mut().add_entry(&format!(
                    "bank correction bounded to envelope: {} -> {} rad",
                    desired_bank, theta,
                ));
            }
            field.set_control_action(theta);
        }

        if let Some(commanded) = field.throttle_action()
            && let Some(bounded) = bounded_throttle
        {
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
