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
//! let pause = CfdFlow::march(&world)
//!     .couple(stack)        // the multiphysics stack (the "what happens each step")
//!     .from(state)          // the MarchState to resume from (its coupled field)
//!     .until(event)?;       // -> CompressiblePause at the event
//! // terminal forms: .run()? (config horizon) / .run_for(n)? (fixed horizon) -> Report
//! ```
//!
//! The blackout trigger and the scalar kappa fold into optional stages (`trigger`, `kappa`) with
//! never-fire / zero defaults; `from` is required, so a coupled march always has an explicit
//! initial field (a fresh one the caller built, or one a pause exported).

use crate::CfdScalar;
use crate::types::flow::{
    BlackoutTrigger, CompressibleMarchRun, CompressiblePause, CoupledField, MarchState,
    PhysicsStage, Report,
};
use crate::types::flow_config::MarchStop;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_haft::LogAddEntry;
use deep_causality_num::FromPrimitive;
use deep_causality_physics::PhysicsError;

/// The field scalar counting leg re-seeds, incremented every time a march resumes from a
/// [`MarchState`]. Cumulative across legs, because the coupled field carries it.
///
/// Read this rather than counting `"leg re-seeded"` substrings in a rendered log: the substring
/// count depends on the message's wording, and a reworded message silently reports zero re-seeds.
pub const LEG_RE_SEEDS_FIELD: &str = "leg_re_seeds";

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
    ///
    /// **A leg boundary carries the coupled field, not the marched fluid layer.** `MarchState` is
    /// the field plus a step index; the incoming leg rebuilds its carrier and re-quantizes the
    /// world's uniform seed, so the evolved conserved state, the inflow strip, any acoustic-envelope
    /// drift the previous leg earned, its rebuild count, and its plume-imprint budget are all
    /// discarded. That is the design's accepted quasi-steady defense — the layer re-converges within
    /// a few steps — but it was previously **invisible**: the fork path logs its resume while this
    /// path logged nothing at all, leaving the most consequential event at a leg seam absent from
    /// provenance. It is recorded here.
    pub fn from(self, state: MarchState<R>) -> ReadyMarch<'c, R, S> {
        let resumed_at = state.step();
        let mut field = state.into_field();
        // The typed counter beside the prose. The count is cumulative across legs because the
        // coupled field carries it, which is what a consumer asking "how many leg boundaries did
        // this descent cross" wants; a per-leg number would answer a different question. Counting
        // rendered log lines instead answers neither reliably, since the count then depends on the
        // message's wording.
        let seeds = field
            .scalar(LEG_RE_SEEDS_FIELD)
            .and_then(|s| s.first().copied())
            .unwrap_or_else(R::zero)
            + R::one();
        field.set_scalar(LEG_RE_SEEDS_FIELD, alloc::vec::Vec::from([seeds]));
        field.log_mut().add_entry(&alloc::format!(
            "leg re-seeded from step {} in world '{}': coupled field carried, marched fluid state re-seeded from the world seed",
            resumed_at,
            self.run.effective_config().name(),
        ));
        ReadyMarch {
            run: self.run,
            coupling: self.coupling,
            field,
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
