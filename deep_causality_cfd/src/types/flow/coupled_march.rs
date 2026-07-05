/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The named-stage coupled march builder: the readable replacement for the positional
//! `run_until(coupling, field, trigger, kappa, predicate)`.
//!
//! A coupled leg reads as named stages instead of a five-argument call:
//!
//! ```ignore
//! let pause = CfdFlow::compressible_march(&world)
//!     .couple(stack)        // the multiphysics stack (the "what happens each step")
//!     .from(state)          // the MarchState to resume from (its coupled field)
//!     .until(event)?;       // -> CompressiblePause at the event
//! // terminal forms: .run()? (config horizon) / .run_for(n)? (fixed horizon) -> Report
//! ```
//!
//! The blackout trigger and the scalar kappa fold into optional stages (`trigger`, `kappa`) with
//! never-fire / zero defaults; `from` is required, so a coupled march always has an explicit
//! initial field (a fresh one the caller built, or one a pause exported).

use crate::types::CfdScalar;
use crate::types::flow::{
    BlackoutTrigger, CompressibleMarchRun, CompressiblePause, CoupledField, MarchState,
    PhysicsStage, Report,
};
use crate::types::flow_config::MarchStop;
use deep_causality_num::{ConjugateScalar, FromPrimitive};
use deep_causality_physics::PhysicsError;

/// A never-firing blackout trigger: a comms threshold so high the plasma sheath never denies the
/// link, the default when a leg does not model blackout.
fn never_fire<R: CfdScalar + FromPrimitive>() -> BlackoutTrigger<R> {
    BlackoutTrigger::new(R::from_f64(1.0e30).expect("scalar represents 1e30"))
}

/// The coupled march with its stack attached, awaiting the initial field. Opened by
/// [`CompressibleMarchRun::couple`](crate::CompressibleMarchRun::couple).
pub struct CoupledMarch<'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    run: CompressibleMarchRun<'c, R>,
    coupling: S,
    trigger: BlackoutTrigger<R>,
    kappa: R,
}

impl<'c, R, S> CoupledMarch<'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    S: PhysicsStage<2, R>,
{
    pub(crate) fn new(run: CompressibleMarchRun<'c, R>, coupling: S) -> Self {
        Self {
            run,
            coupling,
            trigger: never_fire(),
            kappa: R::default(),
        }
    }

    /// Set the blackout trigger (the comms-denial threshold). Defaults to never-fire.
    pub fn trigger(mut self, trigger: BlackoutTrigger<R>) -> Self {
        self.trigger = trigger;
        self
    }

    /// Set the scalar coupling constant `kappa`. Defaults to zero.
    pub fn kappa(mut self, kappa: R) -> Self {
        self.kappa = kappa;
        self
    }

    /// Provide the initial field to march from — a fresh field the caller built, or one a pause
    /// exported through [`MarchState`]. Required before a terminal stage.
    pub fn from(self, state: MarchState<R>) -> ReadyMarch<'c, R, S> {
        ReadyMarch {
            run: self.run,
            coupling: self.coupling,
            field: state.into_field(),
            trigger: self.trigger,
            kappa: self.kappa,
        }
    }

    /// Provide the initial field directly (bypassing [`MarchState`]), for a fresh coupled march
    /// whose field the caller seeded from scratch.
    pub fn from_field(self, field: CoupledField<R>) -> ReadyMarch<'c, R, S> {
        ReadyMarch {
            run: self.run,
            coupling: self.coupling,
            field,
            trigger: self.trigger,
            kappa: self.kappa,
        }
    }
}

/// A coupled march ready to run: stack and initial field attached. Terminal stages produce a
/// [`CompressiblePause`] (`until`) or an owned [`Report`] (`run` / `run_for`).
pub struct ReadyMarch<'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    run: CompressibleMarchRun<'c, R>,
    coupling: S,
    field: CoupledField<R>,
    trigger: BlackoutTrigger<R>,
    kappa: R,
}

impl<'c, R, S> ReadyMarch<'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    S: PhysicsStage<2, R>,
{
    /// March until `predicate` fires (or the horizon is exhausted), yielding a resumable pause.
    ///
    /// # Errors
    /// Solver-assembly or seed-quantization failures; step errors are captured in the pause.
    pub fn until<P>(self, predicate: P) -> Result<CompressiblePause<'c, R, S>, PhysicsError>
    where
        P: Fn(&CoupledField<R>, usize) -> bool,
    {
        self.run.run_until(
            self.coupling,
            self.field,
            self.trigger,
            self.kappa,
            predicate,
        )
    }

    /// March to the config's horizon and report (no pause).
    ///
    /// # Errors
    /// Any assembly, marching, coupling, or reporting failure.
    pub fn run(self) -> Result<Report<R>, PhysicsError> {
        self.run
            .run_coupled(self.coupling, self.field, self.trigger, self.kappa)
    }

    /// March a fixed number of steps and report, overriding the config horizon.
    ///
    /// # Errors
    /// Any assembly, marching, coupling, or reporting failure.
    pub fn run_for(self, steps: usize) -> Result<Report<R>, PhysicsError> {
        self.run.march_with(MarchStop::Fixed(steps)).run_coupled(
            self.coupling,
            self.field,
            self.trigger,
            self.kappa,
        )
    }
}
