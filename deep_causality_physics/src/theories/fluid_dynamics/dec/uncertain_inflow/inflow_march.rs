/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The uncertain-inflow march: a `CausalFlow` time loop whose every step collapses a
//! `MaybeUncertain<R>` sensor reading to a prescribed wall velocity, falling back through a
//! Pearl `do(...)` intervention on a dropout — with the **stateless solver left untouched**.
//!
//! The monad carries the state (design correction: solvers are stateless and portable; the
//! `CausalFlow`/`PropagatingProcess` carries state on demand). Per step the [`inflow_march_step`]
//! stage:
//!
//! 1. presence-gates the step's sensor sample (`lift_to_uncertain`) and, if present, collapses it
//!    to a scalar inflow `R` (`expected_value`), updating the last-good value in `State`;
//! 2. on a dropout (`PresenceError`, or a non-finite mean) substitutes the last-good value and
//!    records it in the `EffectLog` at the zone's [`DropoutVerbosity`];
//! 3. reconfigures the prescribed wall BC to that value through the solver's existing
//!    `with_moving_wall` builder, then calls the unchanged `step(&self, field)`;
//! 4. carries the new divergence-free field forward as `State`, and — on a dropout — marks the
//!    substitution as a value alternation via [`Intervenable::intervene`].
//!
//! The uncertain types never enter the solver core (design D6/C3): the collapse to `R` happens
//! entirely in this stage, above `step`.

use deep_causality_core::{
    CausalFlow, CausalityError, CausalityErrorEnum, EffectLog, EffectValue, Intervenable,
    PropagatingProcess,
};
use deep_causality_haft::LogAddEntry;
use deep_causality_uncertain::{MaybeUncertain, ProbabilisticType, UncertainError};

use crate::error::physics_error::PhysicsError;
use crate::quantities::fluid_dynamics::solenoidal_field::SolenoidalField;
use crate::theories::fluid_dynamics::dec::DecNsScalar;
use crate::theories::fluid_dynamics::dec::dec_ns_solver::DecNsSolver;

use super::dropout_verbosity::DropoutVerbosity;
use super::uncertain_inflow_zone::UncertainInflowZone;

/// Mutable march state (design D10): the stateless solver, the current divergence-free field, the
/// last-good inflow value, the step index, and whether the previous step was a dropout (for
/// transition logging).
///
/// The solver is held in an `Option` only so the consuming `with_moving_wall` reconfiguration can
/// be threaded through the state without a panic on the (driver-prevalidated) error path; on the
/// live march it is always `Some`.
#[derive(Debug)]
pub struct InflowMarchState<'m, const D: usize, R: DecNsScalar> {
    solver: Option<DecNsSolver<'m, D, R>>,
    field: SolenoidalField<R>,
    last_good: R,
    step: usize,
    in_dropout: bool,
}

impl<'m, const D: usize, R: DecNsScalar> InflowMarchState<'m, D, R> {
    /// Seeds the march from a configured solver, an initial divergence-free field, and the
    /// fallback inflow value (the initial last-good).
    pub fn new(
        solver: DecNsSolver<'m, D, R>,
        field: SolenoidalField<R>,
        default_inflow: R,
    ) -> Self {
        Self {
            solver: Some(solver),
            field,
            last_good: default_inflow,
            step: 0,
            in_dropout: false,
        }
    }

    /// The current divergence-free field.
    pub fn field(&self) -> &SolenoidalField<R> {
        &self.field
    }

    /// The last presence-confirmed inflow value (the dropout fallback).
    pub fn last_good(&self) -> R {
        self.last_good
    }

    /// The number of steps marched so far.
    pub fn step(&self) -> usize {
        self.step
    }

    /// Whether the most recent step was a sensor dropout.
    pub fn in_dropout(&self) -> bool {
        self.in_dropout
    }
}

/// Immutable march context (design D10): the zone configuration and the per-step sensor stream.
#[derive(Debug, Clone)]
pub struct InflowContext<R: ProbabilisticType> {
    zone: UncertainInflowZone<R>,
    stream: Vec<MaybeUncertain<R>>,
}

impl<R: ProbabilisticType + Copy> InflowContext<R> {
    /// A context over a zone and a sensor stream (`stream[i]` feeds march step `i`).
    pub fn new(zone: UncertainInflowZone<R>, stream: Vec<MaybeUncertain<R>>) -> Self {
        Self { zone, stream }
    }
}

/// The stateful process the uncertain-inflow march threads: value `R` (the step's inflow),
/// [`InflowMarchState`], [`InflowContext`].
pub type InflowProcess<'m, const D: usize, R> =
    PropagatingProcess<R, InflowMarchState<'m, D, R>, InflowContext<R>>;

/// Builds a short-circuiting error process that preserves the current state and context.
fn error_process<'m, const D: usize, R: DecNsScalar + ProbabilisticType>(
    state: InflowMarchState<'m, D, R>,
    context: Option<InflowContext<R>>,
    message: &str,
) -> InflowProcess<'m, D, R> {
    PropagatingProcess {
        value: EffectValue::None,
        state,
        context,
        error: Some(CausalityError::new(CausalityErrorEnum::Custom(
            message.to_string(),
        ))),
        logs: EffectLog::new(),
    }
}

/// One uncertain-inflow march step (the `CausalFlow` bind stage). See the module docs.
pub fn inflow_march_step<'m, const D: usize, R>(
    _incoming: EffectValue<R>,
    state: InflowMarchState<'m, D, R>,
    context: Option<InflowContext<R>>,
) -> InflowProcess<'m, D, R>
where
    R: DecNsScalar + ProbabilisticType,
{
    let InflowMarchState {
        solver,
        field,
        mut last_good,
        step,
        mut in_dropout,
    } = state;

    // Context and a live solver are required; the stream must cover this step.
    let Some(context) = context else {
        let state = InflowMarchState {
            solver,
            field,
            last_good,
            step,
            in_dropout,
        };
        return error_process(state, None, "uncertain inflow: missing InflowContext");
    };
    let Some(solver) = solver else {
        let state = InflowMarchState {
            solver: None,
            field,
            last_good,
            step,
            in_dropout,
        };
        return error_process(
            state,
            Some(context),
            "uncertain inflow: solver was consumed by a prior failure",
        );
    };
    if step >= context.stream.len() {
        let state = InflowMarchState {
            solver: Some(solver),
            field,
            last_good,
            step,
            in_dropout,
        };
        return error_process(
            state,
            Some(context),
            "uncertain inflow: sensor stream exhausted before the step horizon",
        );
    }

    let zone = context.zone;
    let mut logs = EffectLog::new();

    // 1. Presence-gated collapse of the sensor sample to a scalar inflow R.
    let gate = context.stream[step].lift_to_uncertain(
        zone.threshold(),
        zone.confidence(),
        zone.epsilon(),
        zone.max_samples(),
    );
    let mut dropout = false;
    let inflow = match gate {
        Ok(present) => match present.expected_value(zone.collapse_samples()) {
            Ok(mean) if mean.is_finite() => {
                last_good = mean;
                mean
            }
            // A present-but-degenerate (non-finite) mean is treated as a dropout.
            Ok(_) => {
                dropout = true;
                last_good
            }
            Err(e) => {
                let state = InflowMarchState {
                    solver: Some(solver),
                    field,
                    last_good,
                    step,
                    in_dropout,
                };
                return error_process(
                    state,
                    Some(context),
                    &format!("uncertain inflow: sample reduction failed: {e}"),
                );
            }
        },
        Err(UncertainError::PresenceError(_)) => {
            dropout = true;
            last_good
        }
        Err(e) => {
            let state = InflowMarchState {
                solver: Some(solver),
                field,
                last_good,
                step,
                in_dropout,
            };
            return error_process(
                state,
                Some(context),
                &format!("uncertain inflow: presence gate failed: {e}"),
            );
        }
    };

    // 2. Verbosity-controlled dropout record (the BC-fallback log).
    match zone.verbosity() {
        DropoutVerbosity::EachDropout => {
            if dropout {
                logs.add_entry(&format!(
                    "uncertain-inflow dropout at step {step}: fallback to last-good inflow {last_good:?}"
                ));
            }
        }
        DropoutVerbosity::Transitions => {
            if dropout && !in_dropout {
                logs.add_entry(&format!(
                    "uncertain-inflow dropout ONSET at step {step}: fallback to last-good inflow {last_good:?}"
                ));
            } else if !dropout && in_dropout {
                logs.add_entry(&format!(
                    "uncertain-inflow RECOVERY at step {step}: sensor present again"
                ));
            }
        }
    }
    in_dropout = dropout;

    // 3. Reconfigure the prescribed wall BC to the resolved value, then march. The solver's
    //    `step` is unchanged and stays `&self`; only the boundary value is updated, through the
    //    existing moving-wall lift. Config is prevalidated by the driver and the velocity is
    //    finite, so this reconfiguration does not fail on the live march.
    let mut velocity = [R::zero(); D];
    velocity[zone.flow_axis()] = inflow;
    let solver = match solver.with_moving_wall(zone.wall_axis(), zone.max_side(), velocity) {
        Ok(solver) => solver,
        Err(e) => {
            let state = InflowMarchState {
                solver: None,
                field,
                last_good,
                step,
                in_dropout,
            };
            return error_process(
                state,
                Some(context),
                &format!("uncertain inflow: boundary reconfiguration rejected: {e}"),
            );
        }
    };
    let advanced = match solver.step(&field) {
        Ok(output) => output.into_state(),
        Err(e) => {
            let state = InflowMarchState {
                solver: Some(solver),
                field,
                last_good,
                step,
                in_dropout,
            };
            return error_process(
                state,
                Some(context),
                &format!("uncertain inflow: march step failed: {e}"),
            );
        }
    };

    // 4. Carry the new field forward. On a dropout, the substitution is recorded as a value
    //    alternation: the absent reading (None) is overridden by the fallback via `intervene`.
    let next = InflowMarchState {
        solver: Some(solver),
        field: advanced,
        last_good,
        step: step + 1,
        in_dropout,
    };
    let process = PropagatingProcess {
        value: if dropout {
            EffectValue::None
        } else {
            EffectValue::Value(inflow)
        },
        state: next,
        context: Some(context),
        error: None,
        logs,
    };
    if dropout {
        process.intervene(inflow)
    } else {
        process
    }
}

/// Runs the uncertain-inflow march for `steps` steps and returns the final process — its
/// `EffectLog` holds every recorded dropout, its `State` the final field and last-good value.
///
/// The zone configuration is validated once against the lattice before the march, so the per-step
/// boundary reconfiguration cannot fail on a valid, finite stream.
///
/// # Errors
/// * `PhysicsError::DimensionMismatch` when `wall_axis`/`flow_axis` are out of range, equal, or
///   the stream is shorter than `steps`.
/// * Every rejection of [`DecNsSolver::with_moving_wall`] for the prescribed wall (periodic axis,
///   non-finite or wall-normal velocity).
pub fn march_inflow<'m, const D: usize, R>(
    solver: DecNsSolver<'m, D, R>,
    field: SolenoidalField<R>,
    zone: UncertainInflowZone<R>,
    stream: Vec<MaybeUncertain<R>>,
    steps: usize,
) -> Result<InflowProcess<'m, D, R>, PhysicsError>
where
    R: DecNsScalar + ProbabilisticType,
{
    if zone.flow_axis() >= D {
        return Err(PhysicsError::DimensionMismatch(format!(
            "march_inflow: flow axis {} out of range for D = {D}",
            zone.flow_axis()
        )));
    }
    if stream.len() < steps {
        return Err(PhysicsError::DimensionMismatch(format!(
            "march_inflow: sensor stream has {} samples but the horizon is {steps} steps",
            stream.len()
        )));
    }

    // Validate the prescribed-wall configuration once (axis range, periodicity, and the
    // wall-normal/equal-axis rule) by installing the fallback value as the initial wall lift.
    let mut probe = [R::zero(); D];
    probe[zone.flow_axis()] = zone.default_inflow();
    let solver = solver.with_moving_wall(zone.wall_axis(), zone.max_side(), probe)?;

    let initial = InflowMarchState::new(solver, field, zone.default_inflow());
    let context = InflowContext::new(zone, stream);
    let seed = PropagatingProcess {
        value: EffectValue::Value(zone.default_inflow()),
        state: initial,
        context: Some(context),
        error: None,
        logs: EffectLog::new(),
    };

    let flow = CausalFlow::from(seed).iterate_n(steps, |flow| flow.bind(inflow_march_step));
    Ok(flow.into_process())
}
