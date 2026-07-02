/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **resumable, forkable** QTT march (Stage 4 — the mid-march counterfactual attach point).
//!
//! [`QttMarchRun::run_until`](super::QttMarchRun::run_until) pauses the coupled loop at a predicate
//! and yields a [`MarchPause`]: the borrowed world (config + coupling), the fluid state, and the
//! coupled field at the pause step, with the fluid and field behind `Arc`. A [`MarchPause::fork`] is
//! **O(1)** — it clones the `Arc`s, not the tensor data — so corridor [5] spawns one fork per
//! candidate bank-angle world from the *same* shared blackout-onset state. Each fork alternates its
//! world/state through the verbatim `deep_causality_core` vocabulary (`alternate_context` /
//! `alternate_state` / `alternate_value`, each appending its `!!*Alternation!!` audit entry), then
//! [`MarchFork::continue_march`] rebuilds the solver from the (alternated) config and resumes from
//! the branch state.
//!
//! **Copy-on-write.** The march never mutates fluid trains in place (each step *produces* the next
//! state), so a continued branch reads the shared onset state and replaces its own `Arc` — no tensor
//! data is ever copied. The coupled field *is* mutated (stages write scalars), so the first write
//! triggers exactly one `Arc::make_mut` clone; the pause's copy stays pristine and every further
//! fork sees the identical onset. Read → share, write → CoW.
//!
//! **Error channel.** A step failure inside `run_until` is captured into the pause (with a
//! provenance entry), not thrown: the pause is a carrier. Per the `Alternatable` contract the error
//! channel is never alternated — alternation on an errored fork applies nothing and appends only the
//! audit entry — and `continue_march` on an errored fork returns the captured error.

use super::blackout::BlackoutTrigger;
use super::coupling::{CoupledField, PhysicsStage};
use super::qtt_march_run::{
    BlackoutSampler, QttState, build_solver, coupled_step, finish_coupled_report,
};
use crate::types::CfdScalar;
use crate::types::flow::Report;
use crate::types::flow_config::QttMarchConfig;
use alloc::sync::Arc;
use deep_causality_core::{AlternatableContext, AlternatableState, AlternatableValue, EffectLog};
use deep_causality_haft::{LogAddEntry, LogAppend};
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::CausalTensor;

/// A coupled march paused mid-flight: the shared branch state every counterfactual fork resumes
/// from. Produced by [`QttMarchRun::run_until`](super::QttMarchRun::run_until).
pub struct MarchPause<'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    pub(in crate::types::flow) config: &'c QttMarchConfig<R>,
    pub(in crate::types::flow) coupling: S,
    pub(in crate::types::flow) trigger: BlackoutTrigger<R>,
    pub(in crate::types::flow) scalar_kappa: R,
    pub(in crate::types::flow) state: Arc<QttState<R>>,
    pub(in crate::types::flow) field: Arc<CoupledField<R>>,
    pub(in crate::types::flow) step: usize,
    pub(in crate::types::flow) error: Option<PhysicsError>,
}

impl<'c, R, S> MarchPause<'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// The 1-based step count completed when the march paused.
    pub fn step(&self) -> usize {
        self.step
    }

    /// The captured step error, if the march broke before the predicate fired.
    pub fn error(&self) -> Option<&PhysicsError> {
        self.error.as_ref()
    }

    /// The coupled field at the pause (carried scalars, nav state, regime, provenance log).
    pub fn field(&self) -> &CoupledField<R> {
        &self.field
    }

    /// Fork the pause: an **O(1)** branch sharing the onset fluid state and field by `Arc` — no
    /// tensor data is copied until (and unless) the branch writes.
    pub fn fork(&self) -> MarchFork<'_, 'c, R, S> {
        MarchFork {
            pause: self,
            context_ov: None,
            seed_ov: None,
            state: Arc::clone(&self.state),
            field: Arc::clone(&self.field),
            log: EffectLog::new(),
            error: self.error.clone(),
        }
    }
}

/// One counterfactual branch forked from a [`MarchPause`]: alternate its world or state, then
/// [`continue_march`](Self::continue_march). Alternation uses the verbatim core vocabulary; the
/// error channel is never alternated.
pub struct MarchFork<'p, 'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    pause: &'p MarchPause<'c, R, S>,
    context_ov: Option<&'c QttMarchConfig<R>>,
    seed_ov: Option<(CausalTensor<R>, CausalTensor<R>)>,
    state: Arc<QttState<R>>,
    field: Arc<CoupledField<R>>,
    log: EffectLog,
    error: Option<PhysicsError>,
}

/// `!!ContextAlternation!!` — resume this branch in a different **world** (a whole checked-in
/// config). Not applied on an errored fork (audit entry only).
impl<'p, 'c, R, S> AlternatableContext<&'c QttMarchConfig<R>> for MarchFork<'p, 'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn alternate_context(mut self, new_context: &'c QttMarchConfig<R>) -> Self {
        if self.error.is_some() {
            self.log
                .add_entry("!!ContextAlternation!!: not applied (errored run cannot be repaired)");
            return self;
        }
        self.log.add_entry(&alloc::format!(
            "!!ContextAlternation!!: world '{}' replaced with '{}' at step {}",
            self.pause.config.name(),
            new_context.name(),
            self.pause.step,
        ));
        self.context_ov = Some(new_context);
        self
    }
}

/// `!!StateAlternation!!` — resume this branch from a different fluid state (the carried field is
/// inherited from the pause). Not applied on an errored fork (audit entry only).
impl<'p, 'c, R, S> AlternatableState<(CausalTensor<R>, CausalTensor<R>)> for MarchFork<'p, 'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn alternate_state(mut self, new_state: (CausalTensor<R>, CausalTensor<R>)) -> Self {
        if self.error.is_some() {
            self.log
                .add_entry("!!StateAlternation!!: not applied (errored run cannot be repaired)");
            return self;
        }
        self.log.add_entry(&alloc::format!(
            "!!StateAlternation!!: fluid state replaced at step {}",
            self.pause.step
        ));
        self.seed_ov = Some(new_state);
        self
    }
}

/// `!!ValueAlternation!!` — inject a primary-state snapshot into this branch (the `intervene`
/// analog). Not applied on an errored fork (audit entry only).
impl<'p, 'c, R, S> AlternatableValue<(CausalTensor<R>, CausalTensor<R>)> for MarchFork<'p, 'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn alternate_value(mut self, new_value: (CausalTensor<R>, CausalTensor<R>)) -> Self {
        if self.error.is_some() {
            self.log
                .add_entry("!!ValueAlternation!!: not applied (errored run cannot be repaired)");
            return self;
        }
        self.log.add_entry(&alloc::format!(
            "!!ValueAlternation!!: primary-state snapshot injected at step {}",
            self.pause.step
        ));
        self.seed_ov = Some(new_value);
        self
    }
}

impl<'p, 'c, R, S> MarchFork<'p, 'c, R, S>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    S: PhysicsStage<2, R>,
{
    /// Whether this fork still shares the pause's fluid state by reference (no tensor data copied).
    pub fn shares_fluid_with(&self, pause: &MarchPause<'c, R, S>) -> bool {
        Arc::ptr_eq(&self.state, &pause.state)
    }

    /// Whether this fork still shares the pause's coupled field by reference.
    pub fn shares_field_with(&self, pause: &MarchPause<'c, R, S>) -> bool {
        Arc::ptr_eq(&self.field, &pause.field)
    }

    /// The alternation audit entries this fork has accumulated (merged into the branch's provenance
    /// log on `continue_march`; inspectable directly on an errored fork, whose continue returns the
    /// captured error instead of a report).
    pub fn audit_log(&self) -> &EffectLog {
        &self.log
    }

    /// Resume the march for `steps` further coupled steps: rebuild the solver from the (alternated)
    /// world, resume from the shared branch state, and report the continued segment (blackout
    /// series per the world's observe opt-ins, final fields, and the branch's full provenance log —
    /// pause history + alternation markers + everything the continued stages append).
    ///
    /// The first field write performs the branch's single copy-on-write clone; the pause's copy is
    /// untouched, so further forks of the same pause see the identical onset.
    ///
    /// # Errors
    /// The captured pause error (a broken chain propagates; alternation cannot repair it), or any
    /// solver-assembly / marching / coupling failure in the continued segment.
    pub fn continue_march(self, steps: usize) -> Result<Report<R>, PhysicsError> {
        if let Some(e) = self.error {
            return Err(e);
        }
        let cfg = self.context_ov.unwrap_or(self.pause.config);
        let solver = build_solver(cfg)?;

        // Fluid: an alternated snapshot re-quantizes; otherwise resume the shared trains. The loop
        // below only ever *reads* the current state and replaces the Arc with the freshly produced
        // next state, so the shared onset data is never cloned.
        let mut state: Arc<QttState<R>> = match self.seed_ov {
            Some((u, v)) => Arc::new((
                crate::tensor_bridge::quantize_2d(&u, &cfg.trunc)?,
                crate::tensor_bridge::quantize_2d(&v, &cfg.trunc)?,
            )),
            None => self.state,
        };

        // Field: the branch's one CoW clone happens at the first write (merging the audit log).
        let mut field_arc = self.field;
        let mut branch_log = self.log;
        {
            let field = Arc::make_mut(&mut field_arc);
            field.log_mut().append(&mut branch_log);
            field.log_mut().add_entry(&alloc::format!(
                "march resumed at step {} for {} steps in world '{}'",
                self.pause.step,
                steps,
                cfg.name(),
            ));
        }

        let mut sampler = BlackoutSampler::new(self.pause.trigger);
        for s in 0..steps {
            let field = Arc::make_mut(&mut field_arc);
            let next = coupled_step(
                &solver,
                cfg,
                &state,
                field,
                &self.pause.coupling,
                self.pause.scalar_kappa,
                self.pause.step + s + 1,
            )?;
            state = Arc::new(next);
            sampler.sample(field)?;
        }

        finish_coupled_report(cfg, &cfg.observe, sampler, &state, &field_arc)
    }
}
