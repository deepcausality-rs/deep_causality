/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The crate-internal **carrier seam**: one implementation of the coupled march loop and its
//! resumable/forkable counterfactual machinery, shared by every marcher the Flow DSL hosts.
//!
//! A carrier implements [`CoupledCarrier`]: how to build itself from its config, advance its
//! state, publish per-step projections into the [`CoupledField`], and write its final fields into
//! the report. Everything else — the coupled step ordering, the blackout sampler, `run_coupled`
//! and `run_until` drivers, the [`CarrierPause`]/[`CarrierFork`] pair with copy-on-write sharing,
//! error-channel capture, and the verbatim `deep_causality_core` alternation vocabulary — lives
//! here exactly once, so the subtle contracts cannot drift between carriers. The QTT
//! incompressible host and the compressible host are two implementations of the seam; their
//! public types are aliases over these generics.

use super::blackout::BlackoutTrigger;
use super::coupling::{CoupledField, PhysicsStage, StepContext};
use crate::CfdScalar;
use crate::types::flow::{ForkEconomics, MarchState, Report};
use crate::types::flow_config::QttObserve;
use alloc::sync::Arc;
use deep_causality_core::{AlternatableContext, AlternatableState, AlternatableValue, EffectLog};
use deep_causality_haft::{LogAddEntry, LogAppend, LogSize};
use deep_causality_par::{MaybeParallel, scoped_map};
use deep_causality_physics::{ElectronDensity, PhysicsError};

/// One marcher hosted in the coupled Flow-DSL loop. `D` is the spatial dimension its
/// [`PhysicsStage`] stack composes over.
pub trait CoupledCarrier<const D: usize, R: CfdScalar>: Sized {
    /// The owned configuration container the carrier is built from (context alternation swaps a
    /// whole one).
    type Config;
    /// The marched state (tensor-train form).
    type State: Clone;
    /// The dense seed payload a state/value alternation injects.
    type Seed;

    /// Assemble the carrier from its config (also the fork's rebuild-from-alternated-world path).
    ///
    /// # Errors
    /// Solver-assembly failures.
    fn build(cfg: &Self::Config) -> Result<Self, PhysicsError>;

    /// Encode the config's own seed into the marched state.
    ///
    /// # Errors
    /// Quantization failures.
    fn seed_state(&self, cfg: &Self::Config) -> Result<Self::State, PhysicsError>;

    /// Encode an injected seed payload into the marched state.
    ///
    /// # Errors
    /// Quantization failures.
    fn encode_seed(&self, seed: &Self::Seed) -> Result<Self::State, PhysicsError>;

    /// The fixed step size.
    fn dt(&self) -> R;

    /// Solver rebuilds this carrier has performed. Carriers that never rebuild report zero.
    ///
    /// Exposed so a harness or gate reads a number rather than tallying `"carrier rebuilt at step"`
    /// substrings in a rendered provenance string — an accounting that silently stops working if the
    /// message is ever reworded, and that sees only the logs a caller happens to render.
    fn rebuilds(&self) -> usize {
        0
    }

    /// Advance the marched state one step.
    ///
    /// # Errors
    /// Solver failures (positivity, rounding, operator application).
    fn advance(&self, state: &Self::State) -> Result<Self::State, PhysicsError>;

    /// Publish the per-step projections of `state` into the field (speed and whatever evolved
    /// quantities the carrier exposes) and transport the carried scalar with diffusivity `kappa`.
    ///
    /// # Errors
    /// Codec or transport failures.
    fn publish_and_transport(
        &self,
        state: &Self::State,
        field: &mut CoupledField<R>,
        kappa: R,
    ) -> Result<(), PhysicsError>;

    /// Write the final-state fields into the report.
    ///
    /// # Errors
    /// Codec failures.
    fn finish(&self, state: &Self::State, report: &mut Report<R>) -> Result<(), PhysicsError>;

    /// The peak bond dimension of `state`, for a carrier whose state is a tensor train.
    ///
    /// The default is `None`: a carrier whose state carries no rank has nothing to report, and a
    /// compression gate must fail on the absence rather than substitute the configured cap. Reading
    /// the cap back is a comparison of a constant against itself.
    fn peak_bond(&self, _state: &Self::State) -> Option<usize> {
        None
    }

    /// The case name carried by a config.
    fn config_name(cfg: &Self::Config) -> &str;

    /// The observe opt-ins carried by a config.
    fn config_observe(cfg: &Self::Config) -> QttObserve;

    /// A hook that runs before each advance, with mutable access to the carrier and the field: a
    /// scheduled carrier follows the descent here (updates its inflow, rebuilds its solver past
    /// the drift tolerance, and logs each rebuild to the field's provenance). The default is a
    /// no-op.
    ///
    /// # Errors
    /// Schedule-evaluation or rebuild failures.
    fn pre_step(&mut self, _field: &mut CoupledField<R>, _step: usize) -> Result<(), PhysicsError> {
        Ok(())
    }

    /// One step of the coupled loop: the pre-step hook, advance, publish/transport, then apply
    /// the between-step coupling. An `Err` (solver or coupling) leaves `field` at its mid-step
    /// content for the caller to capture or propagate.
    ///
    /// # Errors
    /// As [`pre_step`](Self::pre_step), [`advance`](Self::advance),
    /// [`publish_and_transport`](Self::publish_and_transport), and the coupling stack.
    fn coupled_step<S: PhysicsStage<D, R>>(
        &mut self,
        state: &Self::State,
        field: &mut CoupledField<R>,
        coupling: &S,
        kappa: R,
        step: usize,
    ) -> Result<Self::State, PhysicsError> {
        self.pre_step(field, step)?;
        let next = self.advance(state)?;
        self.publish_and_transport(&next, field, kappa)?;
        let ctx = StepContext::<D, R>::qtt(self.dt(), step);
        coupling.apply(&ctx, field)?;
        Ok(next)
    }
}

/// The per-step blackout observables a coupled march accumulates: peak `n_e`, plasma frequency,
/// the count of link-denied steps, the peak of the published `"speed"` projection, and the sensed
/// `"heat_flux"` a loads stage publishes.
pub struct BlackoutSampler<R: CfdScalar> {
    trigger: BlackoutTrigger<R>,
    ne: Vec<R>,
    wp: Vec<R>,
    speed: Vec<R>,
    heat: Vec<R>,
    denied_steps: usize,
}

impl<R: CfdScalar> BlackoutSampler<R> {
    pub fn new(trigger: BlackoutTrigger<R>) -> Self {
        Self {
            trigger,
            ne: Vec::new(),
            wp: Vec::new(),
            speed: Vec::new(),
            heat: Vec::new(),
            denied_steps: 0,
        }
    }

    /// Sample the field's peak electron density through the trigger, the peak of the published
    /// `"speed"` projection, and the first cell of the sensed `"heat_flux"`. Each is a no-op when
    /// its field is absent.
    ///
    /// # Errors
    /// Trigger evaluation failures.
    pub fn sample(&mut self, field: &CoupledField<R>) -> Result<(), PhysicsError> {
        if let Some(ne) = field.scalar("n_e") {
            let ne_max = ne
                .iter()
                .copied()
                .fold(R::zero(), |acc, x| if x > acc { x } else { acc });
            let decision = self.trigger.evaluate(ElectronDensity::new(ne_max)?)?;
            self.ne.push(ne_max);
            self.wp.push(decision.plasma_frequency);
            if decision.denied {
                self.denied_steps += 1;
            }
        }
        if let Some(speed) = field.scalar("speed") {
            let peak = speed
                .iter()
                .copied()
                .fold(R::zero(), |acc, x| if x > acc { x } else { acc });
            self.speed.push(peak);
        }
        if let Some(q) = field.scalar("heat_flux").and_then(|q| q.first().copied()) {
            self.heat.push(q);
        }
        Ok(())
    }

    /// Fold the accumulated series into the report per the observe opt-ins.
    ///
    /// # Errors
    /// Numeric-lift failures.
    pub fn into_report(
        self,
        observe: &QttObserve,
        dt: R,
        report: &mut Report<R>,
    ) -> Result<(), PhysicsError> {
        if observe.electron_density {
            report.add_series("n_e", self.ne);
        }
        if observe.plasma_frequency {
            report.add_series("plasma_frequency", self.wp);
        }
        if observe.max_speed {
            report.add_series("max_speed", self.speed);
        }
        if observe.heat_flux {
            report.add_series("heat_flux", self.heat);
        }
        if observe.blackout_dwell {
            let dwell = R::from_usize(self.denied_steps)
                .ok_or_else(|| PhysicsError::NumericalInstability("dwell count lift".into()))?
                * dt;
            report.add_series("blackout_dwell", Vec::from([dwell]));
        }
        Ok(())
    }
}

/// Close a coupled report: fold the blackout series, let the carrier expose its final fields,
/// publish the terminal truth/navigation states the branch scoring reads, and attach the field's
/// provenance log when it carries entries.
fn finish_report<const D: usize, R, M>(
    carrier: &M,
    cfg: &M::Config,
    observe: &QttObserve,
    sampler: BlackoutSampler<R>,
    state: &M::State,
    field: &CoupledField<R>,
) -> Result<Report<R>, PhysicsError>
where
    R: CfdScalar,
    M: CoupledCarrier<D, R>,
{
    let mut report = Report::new(M::config_name(cfg));
    sampler.into_report(observe, carrier.dt(), &mut report)?;
    carrier.finish(state, &mut report)?;
    if let Some(bond) = carrier.peak_bond(state) {
        report.set_peak_bond(bond);
    }
    // Expose the final field's scalars as `final_<name>` so a reduction reads any carried quantity
    // off the report — the terminal trajectory witnesses (the carried truth state and the last
    // published navigation solution, so a branch's miss is trajectory-derived, not modeled) and the
    // coupling's own telemetry (the weather table's blackout-window and drift scalars). The
    // carrier's finish-series (from the decoded state) take precedence on a name clash.
    for (name, data) in field.scalars() {
        let key = alloc::format!("final_{name}");
        if report.series(&key).is_none() {
            report.add_series(key, data.clone());
        }
    }
    if !field.log().is_empty() {
        report.set_effect_log(field.log().clone());
    }
    Ok(report)
}

/// The per-run loop invariants a host hands to a driver: the composed coupling stack, the
/// blackout trigger, the carried-scalar diffusivity, and the step horizon.
pub struct CoupledLoopSpec<R: CfdScalar, S> {
    pub coupling: S,
    pub trigger: BlackoutTrigger<R>,
    pub kappa: R,
    pub steps: usize,
}

/// The audit-log sink seam: after each coupled step the driver flushes the field's newly appended
/// provenance entries here. The no-op [`NoAudit`] is the default — an unaudited run pays nothing
/// and behaves exactly as before; the std `LogSink` (the `save_log` verb) writes each new entry to
/// disk the moment it is recorded, so a killed run's file ends at the last step it completed.
pub trait AuditFlush {
    /// Flush every log entry not yet written. A write failure fails the run: an audited run that
    /// can no longer be audited must not continue silently.
    ///
    /// # Errors
    /// IO failures from the underlying sink.
    fn flush(&mut self, log: &EffectLog) -> Result<(), PhysicsError>;
}

/// The default no-op sink: an unaudited run flushes nothing.
pub struct NoAudit;

impl AuditFlush for NoAudit {
    fn flush(&mut self, _log: &EffectLog) -> Result<(), PhysicsError> {
        Ok(())
    }
}

/// Drive a coupled march to completion: the loop body every host's `run_coupled` delegates to.
///
/// # Errors
/// Any step (solver or coupling) or reporting failure.
pub fn run_coupled_driver<const D: usize, R, S, M>(
    carrier: &mut M,
    cfg: &M::Config,
    spec: CoupledLoopSpec<R, S>,
    mut field: CoupledField<R>,
    mut state: M::State,
    observe: &QttObserve,
    audit: &mut impl AuditFlush,
) -> Result<Report<R>, PhysicsError>
where
    R: CfdScalar,
    S: PhysicsStage<D, R>,
    M: CoupledCarrier<D, R>,
{
    let mut sampler = BlackoutSampler::new(spec.trigger);
    for s in 0..spec.steps {
        state = carrier.coupled_step(&state, &mut field, &spec.coupling, spec.kappa, s + 1)?;
        sampler.sample(&field)?;
        // Stepwise flush: the field's provenance entries this step land on disk before the next
        // step begins, so a killed run's audit file ends at the last completed step.
        audit.flush(field.log())?;
    }
    finish_report(carrier, cfg, observe, sampler, &state, &field)
}

/// Drive a coupled march until a predicate pauses it (or the horizon is exhausted), yielding the
/// resumable [`CarrierPause`]. Step failures are captured into the pause's error channel with a
/// provenance entry; the caller handles assembly failures before calling.
pub fn run_until_driver<'c, const D: usize, R, S, M, P>(
    mut carrier: M,
    cfg: &'c M::Config,
    spec: CoupledLoopSpec<R, S>,
    mut field: CoupledField<R>,
    predicate: P,
    mut state: M::State,
    audit: &mut impl AuditFlush,
) -> Result<CarrierPause<'c, R, S, M, D>, PhysicsError>
where
    R: CfdScalar,
    S: PhysicsStage<D, R>,
    M: CoupledCarrier<D, R>,
    P: Fn(&CoupledField<R>, usize) -> bool,
{
    let mut paused_at = 0usize;
    let mut error = None;
    for s in 0..spec.steps {
        match carrier.coupled_step(&state, &mut field, &spec.coupling, spec.kappa, s + 1) {
            Ok(next) => {
                state = next;
                paused_at = s + 1;
            }
            Err(e) => {
                field.log_mut().add_entry(&alloc::format!(
                    "march error captured at step {}: {e}",
                    s + 1
                ));
                error = Some(e);
                // Flush the captured-error entry before returning the errored pause.
                audit.flush(field.log())?;
                break;
            }
        }
        // Stepwise flush: this step's provenance (regime transitions, rebuilds, nav) hits disk now.
        audit.flush(field.log())?;
        if predicate(&field, paused_at) {
            field
                .log_mut()
                .add_entry(&alloc::format!("march paused at step {paused_at}"));
            audit.flush(field.log())?;
            break;
        }
    }
    let rebuilds = carrier.rebuilds();
    Ok(CarrierPause {
        config: cfg,
        coupling: spec.coupling,
        trigger: spec.trigger,
        scalar_kappa: spec.kappa,
        state: Arc::new(state),
        field: Arc::new(field),
        step: paused_at,
        rebuilds,
        error,
    })
}

/// A coupled march paused mid-flight: the shared branch state every counterfactual fork resumes
/// from. Produced by a host's `run_until`.
pub struct CarrierPause<'c, R, S, M, const D: usize>
where
    R: CfdScalar,
    M: CoupledCarrier<D, R>,
{
    config: &'c M::Config,
    coupling: S,
    trigger: BlackoutTrigger<R>,
    scalar_kappa: R,
    state: Arc<M::State>,
    field: Arc<CoupledField<R>>,
    step: usize,
    rebuilds: usize,
    error: Option<PhysicsError>,
}

impl<'c, R, S, M, const D: usize> CarrierPause<'c, R, S, M, D>
where
    R: CfdScalar + deep_causality_file::BitCodec,
    M: CoupledCarrier<D, R>,
{
    /// Suspend this paused march to disk in one line: the carried field and the step index are
    /// packed into a full resume package (checksummed, fingerprinted) and written to `path`. A
    /// different workflow continues later via
    /// [`load_resume_state`](crate::types::flow::state_snapshot::load_resume_state).
    ///
    /// # Errors
    /// Packing and file failures surface as physics errors naming the cause.
    pub fn save_state_snapshot(
        &self,
        path: impl AsRef<std::path::Path>,
        world_fingerprint: &[u8],
    ) -> Result<(), PhysicsError> {
        crate::types::flow::state_snapshot::save_resume_state(
            path,
            self.field(),
            self.step(),
            world_fingerprint,
        )
    }
}

impl<'c, R, S, M, const D: usize> CarrierPause<'c, R, S, M, D>
where
    R: CfdScalar,
    M: CoupledCarrier<D, R>,
{
    /// The 1-based step count completed when the march paused.
    pub fn step(&self) -> usize {
        self.step
    }

    /// The captured step error, if the march broke before the predicate fired.
    /// Solver rebuilds the carrier performed over this leg.
    ///
    /// Read this rather than tallying `"carrier rebuilt at step"` substrings in a rendered log: the
    /// count is per-carrier, so it is a **per-leg** number, and a carrier is rebuilt at every leg
    /// boundary.
    pub fn rebuilds(&self) -> usize {
        self.rebuilds
    }

    pub fn error(&self) -> Option<&PhysicsError> {
        self.error.as_ref()
    }

    /// Leg re-seeds accumulated on the carried field up to this pause.
    ///
    /// Cumulative across every leg the coupled field has crossed, since the field carries the
    /// counter. Read this rather than counting `"leg re-seeded"` substrings in a rendered log — a
    /// reworded message makes that count report zero.
    ///
    /// Counted in `R` rather than `usize` on purpose. `CfdScalar` requires `FromPrimitive` but not
    /// its converse, so a `usize` return would need a `ToPrimitive` bound that `Float106` does not
    /// satisfy — and the precision alias is a parameter this crate's callers are invited to change.
    /// A count compares against a threshold either way.
    pub fn re_seeds(&self) -> R {
        self.counter(crate::types::flow::LEG_RE_SEEDS_FIELD)
    }

    /// Regime transitions logged on the carried field up to this pause.
    ///
    /// Cumulative across legs, and counted in `R`, for the same reasons as
    /// [`re_seeds`](Self::re_seeds).
    pub fn regime_transitions(&self) -> R {
        self.counter(crate::types::flow::REGIME_TRANSITIONS_FIELD)
    }

    /// A carried monotone counter, zero when the field has never published it.
    fn counter(&self, name: &str) -> R {
        self.field
            .scalar(name)
            .and_then(|s| s.first().copied())
            .unwrap_or_else(R::zero)
    }

    /// The coupled field at the pause (carried scalars, nav state, regime, provenance log).
    pub fn field(&self) -> &CoupledField<R> {
        &self.field
    }

    /// Export this pause as a resumable [`MarchState`]: the carried field plus the step reached.
    /// The state resumes a continued march (in memory now, or from disk later after
    /// [`save`](MarchState::save)) bit-identically to continuing this pause directly.
    pub fn state(&self) -> MarchState<R> {
        MarchState::at((*self.field).clone(), self.step)
    }

    /// Fork the pause: an **O(1)** branch sharing the paused state and field by `Arc` — no tensor
    /// data is copied until (and unless) the branch writes.
    pub fn fork(&self) -> CarrierFork<'_, 'c, R, S, M, D> {
        CarrierFork {
            pause: self,
            context_ov: None,
            seed_ov: None,
            state: Arc::clone(&self.state),
            field: Arc::clone(&self.field),
            log: EffectLog::new(),
            error: self.error.clone(),
            alternation_applied: None,
        }
    }
}

impl<'c, R, S, M, const D: usize> CarrierPause<'c, R, S, M, D>
where
    R: CfdScalar,
    S: PhysicsStage<D, R>,
    M: CoupledCarrier<D, R>,
{
    /// The counterfactual fan-out: fork once per world, alternate each fork into its world, and
    /// continue every branch for `steps` coupled steps. Reports come back in world order, and
    /// each branch carries the same `!!ContextAlternation!!` audit entry a manual
    /// [`fork`](Self::fork) chain would produce.
    ///
    /// Branches are data-independent by construction. A fork shares the paused state and field
    /// through `Arc` and takes its single copy-on-write clone at the first write, so with the
    /// `parallel` feature the branches run concurrently on scoped threads
    /// ([`deep_causality_par::scoped_map`]) and the results are bit-identical to the sequential
    /// order; without the feature the fan-out runs inline. The `MaybeParallel` bounds are vacuous
    /// on serial builds.
    ///
    /// # Errors
    /// The first failing branch's error, in world order. Every branch runs to completion first;
    /// a failure does not cancel its siblings.
    pub fn continue_branches(
        &self,
        worlds: &[&M::Config],
        steps: usize,
    ) -> Result<Vec<Report<R>>, PhysicsError>
    where
        Self: MaybeParallel,
        M::Config: MaybeParallel,
        Report<R>: MaybeParallel,
    {
        scoped_map(worlds, |world| self.continue_with(*world, steps))
            .into_iter()
            .collect()
    }

    /// Fly **one** branch world from this pause: fork (O(1), copy-on-write), alternate into the
    /// world, and continue for `steps`. The singular sibling of [`continue_branches`] — one
    /// world, one continued report, carrying the same `!!ContextAlternation!!` audit entry — for
    /// a study that scores branch worlds one at a time (the campaign's event-fork `branch`
    /// lowers each case onto this).
    ///
    /// The world is borrowed only for the duration of this call: it is consumed to rebuild the
    /// carrier and never escapes into the returned report, so it need **not** outlive the pause's
    /// own config (`'c`). This is what lets a campaign bind branch worlds it owns — one per case,
    /// living only in the study phase — and continue each one without pinning it to wherever the
    /// pause was created.
    ///
    /// # Errors
    /// The captured pause error, or any carrier-assembly / marching / coupling failure in the
    /// continued segment.
    pub fn continue_with(
        &self,
        world: &M::Config,
        steps: usize,
    ) -> Result<Report<R>, PhysicsError> {
        // Mirror `fork().alternate_context(world).continue_march(steps)` exactly, but with `world`
        // as a short-lived borrow: an errored pause returns its captured error untouched, and a
        // healthy one accumulates the same `!!ContextAlternation!!` marker into a fresh branch log
        // before the continued segment merges it into the branch's copy-on-write field.
        if let Some(e) = &self.error {
            return Err(e.clone());
        }
        let mut branch_log = EffectLog::new();
        branch_log.add_entry(&alloc::format!(
            "!!ContextAlternation!!: world '{}' replaced with '{}' at step {}",
            M::config_name(self.config),
            M::config_name(world),
            self.step,
        ));
        // The O(1) fork, and the record of it. Taken from the clones actually handed to the branch,
        // not asserted about them: if this path ever deep-copies instead of sharing, the recorded
        // economics say so and a study gating on `is_o1` fails rather than passing on a stale claim.
        let branch_state = Arc::clone(&self.state);
        let branch_field = Arc::clone(&self.field);
        let economics = ForkEconomics::new(
            Arc::ptr_eq(&branch_state, &self.state),
            Arc::ptr_eq(&branch_field, &self.field),
            Arc::strong_count(&self.state),
            Arc::strong_count(&self.field),
        );
        let mut report = run_continued_segment(
            self,
            world,
            None,
            branch_state,
            branch_field,
            branch_log,
            steps,
        )?;
        report.set_fork_economics(economics);
        // This path always applies the alternation: it returns early on an errored pause above, so
        // reaching here means the branch flew `world`.
        report.set_alternation_applied(true);
        Ok(report)
    }
}

/// One counterfactual branch forked from a [`CarrierPause`]: alternate its world or state, then
/// [`continue_march`](Self::continue_march). Alternation uses the verbatim core vocabulary; the
/// error channel is never alternated.
pub struct CarrierFork<'p, 'c, R, S, M, const D: usize>
where
    R: CfdScalar,
    M: CoupledCarrier<D, R>,
{
    pause: &'p CarrierPause<'c, R, S, M, D>,
    context_ov: Option<&'c M::Config>,
    seed_ov: Option<M::Seed>,
    state: Arc<M::State>,
    field: Arc<CoupledField<R>>,
    log: EffectLog,
    error: Option<PhysicsError>,
    /// Whether a context alternation was applied to this fork, recorded typed beside the log entry
    /// so a consumer never has to distinguish the applied and refused entries by their wording.
    alternation_applied: Option<bool>,
}

/// `!!ContextAlternation!!` — resume this branch in a different **world** (a whole checked-in
/// config). Not applied on an errored fork (audit entry only).
impl<'p, 'c, R, S, M, const D: usize> AlternatableContext<&'c M::Config>
    for CarrierFork<'p, 'c, R, S, M, D>
where
    R: CfdScalar,
    M: CoupledCarrier<D, R>,
{
    fn alternate_context(mut self, new_context: &'c M::Config) -> Self {
        if self.error.is_some() {
            self.log
                .add_entry("!!ContextAlternation!!: not applied (errored run cannot be repaired)");
            // Recorded as a typed refusal beside the prose. The refusal entry carries the same
            // `!!ContextAlternation!!` marker as an applied alternation, so a consumer matching the
            // marker as a substring reads a refused branch as an alternated one.
            self.alternation_applied = Some(false);
            return self;
        }
        self.alternation_applied = Some(true);
        self.log.add_entry(&alloc::format!(
            "!!ContextAlternation!!: world '{}' replaced with '{}' at step {}",
            M::config_name(self.pause.config),
            M::config_name(new_context),
            self.pause.step,
        ));
        self.context_ov = Some(new_context);
        self
    }
}

/// `!!StateAlternation!!` — resume this branch from a different marched state (the carried field
/// is inherited from the pause). Not applied on an errored fork (audit entry only).
impl<'p, 'c, R, S, M, const D: usize> AlternatableState<M::Seed> for CarrierFork<'p, 'c, R, S, M, D>
where
    R: CfdScalar,
    M: CoupledCarrier<D, R>,
{
    fn alternate_state(mut self, new_state: M::Seed) -> Self {
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
impl<'p, 'c, R, S, M, const D: usize> AlternatableValue<M::Seed> for CarrierFork<'p, 'c, R, S, M, D>
where
    R: CfdScalar,
    M: CoupledCarrier<D, R>,
{
    fn alternate_value(mut self, new_value: M::Seed) -> Self {
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

impl<'p, 'c, R, S, M, const D: usize> CarrierFork<'p, 'c, R, S, M, D>
where
    R: CfdScalar,
    S: PhysicsStage<D, R>,
    M: CoupledCarrier<D, R>,
{
    /// Whether this fork still shares the pause's marched state by reference (no tensor data
    /// copied).
    pub fn shares_fluid_with(&self, pause: &CarrierPause<'c, R, S, M, D>) -> bool {
        Arc::ptr_eq(&self.state, &pause.state)
    }

    /// Whether this fork still shares the pause's coupled field by reference.
    pub fn shares_field_with(&self, pause: &CarrierPause<'c, R, S, M, D>) -> bool {
        Arc::ptr_eq(&self.field, &pause.field)
    }

    /// The alternation audit entries this fork has accumulated (merged into the branch's
    /// provenance log on `continue_march`; inspectable directly on an errored fork, whose
    /// continue returns the captured error instead of a report).
    pub fn audit_log(&self) -> &EffectLog {
        &self.log
    }

    /// Resume the march for `steps` further coupled steps: rebuild the carrier from the
    /// (alternated) world, resume from the shared branch state, and report the continued segment
    /// (series per the world's observe opt-ins, final fields, and the branch's full provenance
    /// log — pause history + alternation markers + everything the continued stages append).
    ///
    /// The first field write performs the branch's single copy-on-write clone; the pause's copy is
    /// untouched, so further forks of the same pause see the identical paused state.
    ///
    /// # Errors
    /// The captured pause error (a broken chain propagates; alternation cannot repair it), or any
    /// carrier-assembly / marching / coupling failure in the continued segment.
    pub fn continue_march(self, steps: usize) -> Result<Report<R>, PhysicsError> {
        if let Some(e) = self.error {
            return Err(e);
        }
        // Both the alternated override and the pause's own config share `'c`, so this `unwrap_or`
        // unifies cleanly; the borrowed-world `continue_with` path relaxes that lifetime by calling
        // `run_continued_segment` with a short-lived world directly.
        let cfg = self.context_ov.unwrap_or(self.pause.config);
        let alternation_applied = self.alternation_applied;
        // Same record as the `continue_with` path, read off this fork's own shares.
        let economics = ForkEconomics::new(
            Arc::ptr_eq(&self.state, &self.pause.state),
            Arc::ptr_eq(&self.field, &self.pause.field),
            Arc::strong_count(&self.state),
            Arc::strong_count(&self.field),
        );
        let mut report = run_continued_segment(
            self.pause,
            cfg,
            self.seed_ov,
            self.state,
            self.field,
            self.log,
            steps,
        )?;
        report.set_fork_economics(economics);
        if let Some(applied) = alternation_applied {
            report.set_alternation_applied(applied);
        }
        Ok(report)
    }
}

/// The continued-march segment shared by [`CarrierFork::continue_march`] (borrowed override world)
/// and [`CarrierPause::continue_with`] (short-lived owned world). Taking `cfg` as a plain borrow —
/// decoupled from the pause's `'c` — is exactly what lets a study continue in a world it owns: the
/// world is consumed to rebuild the carrier and to name the world in the audit trail, and never
/// escapes into the returned report.
fn run_continued_segment<const D: usize, R, S, M>(
    pause: &CarrierPause<'_, R, S, M, D>,
    cfg: &M::Config,
    seed_ov: Option<M::Seed>,
    state: Arc<M::State>,
    field: Arc<CoupledField<R>>,
    mut branch_log: EffectLog,
    steps: usize,
) -> Result<Report<R>, PhysicsError>
where
    R: CfdScalar,
    S: PhysicsStage<D, R>,
    M: CoupledCarrier<D, R>,
{
    let mut carrier = M::build(cfg)?;

    // Marched state: an alternated snapshot re-encodes; otherwise resume the shared state. The
    // loop below only ever *reads* the current state and replaces the Arc with the freshly
    // produced next state, so the shared paused data is never cloned.
    let mut state: Arc<M::State> = match seed_ov {
        Some(seed) => Arc::new(carrier.encode_seed(&seed)?),
        None => state,
    };

    // Field: the branch's one CoW clone happens at the first write (merging the audit log).
    let mut field_arc = field;
    {
        let field = Arc::make_mut(&mut field_arc);
        field.log_mut().append(&mut branch_log);
        field.log_mut().add_entry(&alloc::format!(
            "march resumed at step {} for {} steps in world '{}'",
            pause.step,
            steps,
            M::config_name(cfg),
        ));
    }

    let mut sampler = BlackoutSampler::new(pause.trigger);
    for s in 0..steps {
        let field = Arc::make_mut(&mut field_arc);
        let next = carrier.coupled_step(
            &state,
            field,
            &pause.coupling,
            pause.scalar_kappa,
            pause.step + s + 1,
        )?;
        state = Arc::new(next);
        sampler.sample(field)?;
    }

    finish_report(
        &carrier,
        cfg,
        &M::config_observe(cfg),
        sampler,
        &state,
        &field_arc,
    )
}
