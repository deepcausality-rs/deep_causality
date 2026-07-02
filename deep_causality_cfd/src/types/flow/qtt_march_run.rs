/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **CfdFlow** DSL marching pipeline for the QTT 2-D incompressible solver — the tensor-train
//! sibling of [`MarchRun`](crate::MarchRun).
//!
//! `CfdFlow::qtt_march(&config)` borrows a [`QttMarchConfig`] and yields a runnable [`QttMarchRun`].
//! There is no `.on(geometry)` stage — the QTT solver carries no borrowed manifold; the power-of-two
//! grid lives in the config. The terminal `run` / `run_with` quantizes the seed, marches
//! [`QttIncompressible2d`] under the configured stop, samples the enabled tensor-train-native
//! observables each step into an owned [`Report`], and exposes the dequantized final `(u, v)` fields.
//! The pipeline adds **no numerics**: its result matches `QttIncompressible2d::run` for the same seed,
//! horizon, and round policy.

use super::blackout::BlackoutTrigger;
use super::coupling::{CoupledField, PhysicsStage, StepContext};
use super::qtt_march_pause::MarchPause;
use crate::solvers::{
    QttImmersed2d, QttIncompressible2d, divergence_residual, drag_lift, kinetic_energy, max_bond,
    max_speed,
};
use crate::tensor_bridge::{QttProjector2d, dequantize_2d, quantize_2d};
use crate::traits::Marcher;
use crate::types::CfdScalar;
use crate::types::flow::Report;
use crate::types::flow_config::{MarchStop, QttMarchConfig, QttObserve};
use alloc::sync::Arc;
use alloc::vec::Vec;
use deep_causality_core::{AlternatableContext, AlternatableState, AlternatableValue, EffectLog};
use deep_causality_haft::{LogAddEntry, LogAppend, LogSize};
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::{ElectronDensity, PhysicsError};
use deep_causality_tensor::{CausalTensor, CausalTensorTrain};

/// The `(u, v)` velocity train pair the QTT marcher carries.
pub(in crate::types::flow) type QttState<R> = (CausalTensorTrain<R>, CausalTensorTrain<R>);

/// The marcher behind the pipeline: the body-free solver, or the Brinkman-penalized immersed solver
/// when the config carries a body. Both share the `(u, v)` state and expose a projector.
pub(in crate::types::flow) enum QttSolver<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    Free(QttIncompressible2d<R>),
    Body(QttImmersed2d<R>),
}

impl<R> QttSolver<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn advance(&self, state: &QttState<R>) -> Result<QttState<R>, PhysicsError> {
        match self {
            QttSolver::Free(s) => s.advance(state, &()),
            QttSolver::Body(s) => s.advance(state, &()),
        }
    }

    fn projector(&self) -> &QttProjector2d<R> {
        match self {
            QttSolver::Free(s) => s.projector(),
            QttSolver::Body(s) => s.projector(),
        }
    }

    /// Transport (advect + diffuse) a passive scalar train one step. The penalized immersed solver
    /// drives the scalar toward `t_wall` inside the body; the body-free solver has no penalization,
    /// so the scalar is carried unchanged (the coupling still updates it pointwise).
    fn advance_scalar(
        &self,
        scalar: &CausalTensorTrain<R>,
        u: &CausalTensorTrain<R>,
        v: &CausalTensorTrain<R>,
        t_wall: R,
        kappa: R,
    ) -> Result<CausalTensorTrain<R>, PhysicsError> {
        match self {
            QttSolver::Body(s) => s.advance_scalar(scalar, u, v, t_wall, kappa),
            QttSolver::Free(_) => Ok(scalar.clone()),
        }
    }
}

/// Assemble the QTT solver a config describes (the immersed Brinkman solver with a body, the
/// body-free solver without). Shared by every march entry point (`run`, `run_coupled`, `run_until`,
/// and a fork's `continue_march` — which rebuilds from a possibly *alternated* config).
pub(in crate::types::flow) fn build_solver<R>(
    cfg: &QttMarchConfig<R>,
) -> Result<QttSolver<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    match &cfg.body {
        Some(b) => Ok(QttSolver::Body(QttImmersed2d::new(
            cfg.lx,
            cfg.ly,
            cfg.dx,
            cfg.dy,
            cfg.dt,
            cfg.nu,
            b.mask.clone(),
            b.ubx,
            b.uby,
            b.eta,
            cfg.trunc,
        )?)),
        None => Ok(QttSolver::Free(QttIncompressible2d::new(
            cfg.lx, cfg.ly, cfg.dx, cfg.dy, cfg.dt, cfg.nu, cfg.trunc,
        )?)),
    }
}

/// One step of the **coupled** march loop, shared verbatim by `run_coupled`, `run_until`, and a
/// fork's `continue_march`: advance the fluid, publish the per-cell `"speed"` projection, transport
/// the carried `"alpha"` fraction as a tensor train, then apply the between-step coupling. Returns
/// the advanced fluid state; an `Err` (solver or coupling) leaves `field` at its mid-step content
/// for the caller to capture or propagate.
pub(in crate::types::flow) fn coupled_step<R, S>(
    solver: &QttSolver<R>,
    cfg: &QttMarchConfig<R>,
    state: &QttState<R>,
    field: &mut CoupledField<R>,
    coupling: &S,
    scalar_kappa: R,
    step: usize,
) -> Result<QttState<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    S: PhysicsStage<2, R>,
{
    let next = solver.advance(state)?;

    // Publish the per-cell speed projection from the dequantized state.
    let uf = dequantize_2d(&next.0, cfg.lx, cfg.ly)?;
    let vf = dequantize_2d(&next.1, cfg.lx, cfg.ly)?;
    let speed: Vec<R> = uf
        .as_slice()
        .iter()
        .zip(vf.as_slice())
        .map(|(&a, &b)| (a * a + b * b).sqrt())
        .collect();
    field.set_scalar("speed", speed);

    // Transport the carried reacting fraction as a tensor train (advect + diffuse), then let the
    // coupling react (operator split: transport, then relax).
    let carried = field.scalar("alpha").map(|s| s.to_vec());
    if let Some(alpha) = carried {
        let shape = [1usize << cfg.lx, 1usize << cfg.ly];
        let alpha_ct = CausalTensor::new(alpha, shape.to_vec()).map_err(|e| {
            PhysicsError::DimensionMismatch(alloc::format!("alpha quantize: {e:?}"))
        })?;
        let alpha_tt = quantize_2d(&alpha_ct, &cfg.trunc)?;
        let advected =
            solver.advance_scalar(&alpha_tt, &next.0, &next.1, R::zero(), scalar_kappa)?;
        let adv_ct = dequantize_2d(&advected, cfg.lx, cfg.ly)?;
        field.set_scalar("alpha", adv_ct.as_slice().to_vec());
    }

    let ctx = StepContext::<2, R>::qtt(cfg.dt, step);
    coupling.apply(&ctx, field)?;
    Ok(next)
}

/// The per-step blackout observables a coupled march accumulates (peak `n_e`, plasma frequency, and
/// the count of link-denied steps).
pub(in crate::types::flow) struct BlackoutSampler<R: CfdScalar> {
    trigger: BlackoutTrigger<R>,
    ne: Vec<R>,
    wp: Vec<R>,
    denied_steps: usize,
}

impl<R: CfdScalar> BlackoutSampler<R> {
    pub(in crate::types::flow) fn new(trigger: BlackoutTrigger<R>) -> Self {
        Self {
            trigger,
            ne: Vec::new(),
            wp: Vec::new(),
            denied_steps: 0,
        }
    }

    /// Sample the field's peak electron density through the trigger (a no-op without `"n_e"`).
    pub(in crate::types::flow) fn sample(
        &mut self,
        field: &CoupledField<R>,
    ) -> Result<(), PhysicsError> {
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
        Ok(())
    }

    /// Fold the accumulated series into the report per the observe opt-ins.
    pub(in crate::types::flow) fn into_report(
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
        if observe.blackout_dwell {
            let dwell = R::from_usize(self.denied_steps)
                .ok_or_else(|| PhysicsError::NumericalInstability("dwell count lift".into()))?
                * dt;
            report.add_series("blackout_dwell", Vec::from([dwell]));
        }
        Ok(())
    }
}

/// Close a coupled report: fold the blackout series, expose the final `(u, v)` fields, and attach
/// the field's provenance log ([7]) when it carries entries.
pub(in crate::types::flow) fn finish_coupled_report<R>(
    cfg: &QttMarchConfig<R>,
    observe: &QttObserve,
    sampler: BlackoutSampler<R>,
    state: &QttState<R>,
    field: &CoupledField<R>,
) -> Result<Report<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let mut report = Report::new(cfg.name.clone());
    sampler.into_report(observe, cfg.dt, &mut report)?;
    let uf = dequantize_2d(&state.0, cfg.lx, cfg.ly)?;
    let vf = dequantize_2d(&state.1, cfg.lx, cfg.ly)?;
    report.set_final_field(uf.as_slice().to_vec());
    report.add_series("final_v", vf.as_slice().to_vec());
    if !field.log().is_empty() {
        report.set_effect_log(field.log().clone());
    }
    Ok(report)
}

/// A geometry-free, runnable QTT marching pipeline. The overrides (`seed_with` / `march_with` /
/// `observe_with`) swap one spec for a counterfactual while reusing the borrowed container.
///
/// # Counterfactual alternation (the pre-run attach point)
///
/// The run implements the `deep_causality_core` alternation vocabulary **verbatim** — a call site
/// that simulates an alternate reality says so loudly:
/// * [`alternate_context`](AlternatableContext::alternate_context) swaps the **whole world** — a
///   different borrowed [`QttMarchConfig`] (a checked-in named constructor, not a delta).
/// * [`alternate_state`](AlternatableState::alternate_state) swaps the **marching state** — the
///   seed `(u0, v0)` pair (this subsumes [`seed_with`](Self::seed_with)).
/// * [`alternate_value`](AlternatableValue::alternate_value) injects a **primary-state snapshot**
///   (the `intervene` analog; at the pre-run attach point it lands on the seed).
///
/// Each verb appends its `!!*Alternation!!` audit entry to the run's provenance log, which a coupled
/// run threads into the [`CoupledField`] log and surfaces on the [`Report`]. The error channel is
/// never alternated (pre-run there is none; a fork honors the same contract explicitly).
pub struct QttMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    config: &'c QttMarchConfig<R>,
    context_ov: Option<&'c QttMarchConfig<R>>,
    seed_ov: Option<(
        deep_causality_tensor::CausalTensor<R>,
        deep_causality_tensor::CausalTensor<R>,
    )>,
    stop_ov: Option<MarchStop<R>>,
    observe_ov: Option<QttObserve>,
    log: EffectLog,
}

/// `!!ContextAlternation!!` — swap the whole world (a different borrowed config) before marching.
impl<'c, R> AlternatableContext<&'c QttMarchConfig<R>> for QttMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn alternate_context(mut self, new_context: &'c QttMarchConfig<R>) -> Self {
        self.log.add_entry(&alloc::format!(
            "!!ContextAlternation!!: world '{}' replaced with '{}'",
            self.effective_config().name(),
            new_context.name()
        ));
        self.context_ov = Some(new_context);
        self
    }
}

/// `!!StateAlternation!!` — swap the marching state (the seed pair) before marching.
impl<'c, R> AlternatableState<(CausalTensor<R>, CausalTensor<R>)> for QttMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn alternate_state(mut self, new_state: (CausalTensor<R>, CausalTensor<R>)) -> Self {
        self.log
            .add_entry("!!StateAlternation!!: marching seed replaced");
        self.seed_ov = Some(new_state);
        self
    }
}

/// `!!ValueAlternation!!` — inject a primary-state snapshot (the `intervene` analog); pre-run it
/// lands on the seed.
impl<'c, R> AlternatableValue<(CausalTensor<R>, CausalTensor<R>)> for QttMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn alternate_value(mut self, new_value: (CausalTensor<R>, CausalTensor<R>)) -> Self {
        self.log
            .add_entry("!!ValueAlternation!!: primary-state snapshot injected");
        self.seed_ov = Some(new_value);
        self
    }
}

impl<'c, R> QttMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Inject a QTT marching container (called by [`CfdFlow::qtt_march`](crate::CfdFlow)).
    pub(crate) fn new(config: &'c QttMarchConfig<R>) -> Self {
        Self {
            config,
            context_ov: None,
            seed_ov: None,
            stop_ov: None,
            observe_ov: None,
            log: EffectLog::new(),
        }
    }

    /// The world this run marches in: the alternated context if one was swapped in, else the
    /// injected config.
    fn effective_config(&self) -> &'c QttMarchConfig<R> {
        self.context_ov.unwrap_or(self.config)
    }

    /// Override the seed velocity fields for this run (counterfactual).
    pub fn seed_with(
        mut self,
        u0: deep_causality_tensor::CausalTensor<R>,
        v0: deep_causality_tensor::CausalTensor<R>,
    ) -> Self {
        self.seed_ov = Some((u0, v0));
        self
    }

    /// Override the march-stop for this run.
    pub fn march_with(mut self, stop: MarchStop<R>) -> Self {
        self.stop_ov = Some(stop);
        self
    }

    /// Override the observe set for this run.
    pub fn observe_with(mut self, observe: QttObserve) -> Self {
        self.observe_ov = Some(observe);
        self
    }

    /// Run the composed workflow, returning the owned report.
    ///
    /// # Errors
    /// Any solver-assembly, quantization, marching, or observable-extraction failure.
    pub fn run(self) -> Result<Report<R>, PhysicsError> {
        self.execute(None::<fn(&QttStepView<'_, R>)>)
    }

    /// Run with a per-step `hook` (called after every projected step with a [`QttStepView`]) — for
    /// progress lines or streamed diagnostics. The series and the final report are identical to
    /// [`run`](Self::run).
    ///
    /// # Errors
    /// As [`run`](Self::run).
    pub fn run_with<H: FnMut(&QttStepView<'_, R>)>(
        self,
        hook: H,
    ) -> Result<Report<R>, PhysicsError> {
        self.execute(Some(hook))
    }

    /// Run the QTT march with a between-step **coupling** hosted in the loop (design D5/D8): each step
    /// publishes a per-cell `"speed"` projection from the dequantized state, transports the carried
    /// reacting fraction (`"alpha"`) via the solver's `advance_scalar` (it stays a tensor train),
    /// applies the coupling, and samples the blackout observables (`n_e`, plasma frequency, dwell)
    /// into the report per the `QttObserve` opt-in flags. The solver's spectral-projection / Brinkman
    /// `advance` is unchanged. `trigger` maps the peak electron density to the blackout decision;
    /// `scalar_kappa` is the passive-scalar diffusivity used to transport the carried fraction.
    ///
    /// # Errors
    /// Any solver-assembly, quantization, marching, coupling, or observable-extraction failure.
    pub fn run_coupled<S>(
        self,
        coupling: S,
        initial: CoupledField<R>,
        trigger: BlackoutTrigger<R>,
        scalar_kappa: R,
    ) -> Result<Report<R>, PhysicsError>
    where
        S: PhysicsStage<2, R>,
    {
        let cfg = self.effective_config();
        let observe = self.observe_ov.unwrap_or(cfg.observe);
        let stop = self.stop_ov.unwrap_or(cfg.stop);
        let (u0, v0) = match self.seed_ov {
            Some(seed) => seed,
            None => (cfg.u0.clone(), cfg.v0.clone()),
        };

        let solver = build_solver(cfg)?;
        let mut state: QttState<R> = (quantize_2d(&u0, &cfg.trunc)?, quantize_2d(&v0, &cfg.trunc)?);

        let mut field = initial;
        // Thread the pre-run alternation markers into the field's provenance log.
        let mut pre_log = self.log;
        field.log_mut().append(&mut pre_log);
        let mut sampler = BlackoutSampler::new(trigger);

        let steps = match stop {
            MarchStop::Fixed(n) => n,
            MarchStop::Steady { max_steps, .. } => max_steps,
        };

        for s in 0..steps {
            state = coupled_step(
                &solver,
                cfg,
                &state,
                &mut field,
                &coupling,
                scalar_kappa,
                s + 1,
            )?;
            sampler.sample(&field)?;
        }

        finish_coupled_report(cfg, &observe, sampler, &state, &field)
    }

    /// March the coupled loop **until a predicate pauses it** (or the stop horizon is exhausted),
    /// yielding a resumable [`MarchPause`] instead of a final report — the mid-march attach point of
    /// the counterfactual study (corridor [5]: fork the pause once per candidate world, alternate,
    /// and continue each branch from the *same* shared onset state).
    ///
    /// The predicate is checked after each coupled step against the mutated field (e.g. "blackout
    /// onset": the classifier's denial flag just went up). A **step** failure (solver or coupling)
    /// does not tear the march down: it is captured into the pause's error channel with a provenance
    /// entry, honoring the `Alternatable` contract that a broken chain propagates its error (a fork
    /// of an errored pause cannot be repaired by alternation). Assembly failures (solver build, seed
    /// quantization) fail fast with `Err` — there is no state to pause.
    ///
    /// # Errors
    /// Solver-assembly or seed-quantization failures only; step errors are captured in the pause.
    pub fn run_until<S, P>(
        self,
        coupling: S,
        initial: CoupledField<R>,
        trigger: BlackoutTrigger<R>,
        scalar_kappa: R,
        predicate: P,
    ) -> Result<MarchPause<'c, R, S>, PhysicsError>
    where
        S: PhysicsStage<2, R>,
        P: Fn(&CoupledField<R>, usize) -> bool,
    {
        let cfg = self.effective_config();
        let stop = self.stop_ov.unwrap_or(cfg.stop);
        let (u0, v0) = match self.seed_ov {
            Some(seed) => seed,
            None => (cfg.u0.clone(), cfg.v0.clone()),
        };

        let solver = build_solver(cfg)?;
        let mut state: QttState<R> = (quantize_2d(&u0, &cfg.trunc)?, quantize_2d(&v0, &cfg.trunc)?);

        let mut field = initial;
        let mut pre_log = self.log;
        field.log_mut().append(&mut pre_log);

        let steps = match stop {
            MarchStop::Fixed(n) => n,
            MarchStop::Steady { max_steps, .. } => max_steps,
        };

        let mut paused_at = 0usize;
        let mut error = None;
        for s in 0..steps {
            match coupled_step(
                &solver,
                cfg,
                &state,
                &mut field,
                &coupling,
                scalar_kappa,
                s + 1,
            ) {
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
                    break;
                }
            }
            if predicate(&field, paused_at) {
                field
                    .log_mut()
                    .add_entry(&alloc::format!("march paused at step {paused_at}"));
                break;
            }
        }

        Ok(MarchPause {
            config: cfg,
            coupling,
            trigger,
            scalar_kappa,
            state: Arc::new(state),
            field: Arc::new(field),
            step: paused_at,
            error,
        })
    }

    fn execute<H: FnMut(&QttStepView<'_, R>)>(
        self,
        mut hook: Option<H>,
    ) -> Result<Report<R>, PhysicsError> {
        let cfg = self.effective_config();
        let observe = self.observe_ov.unwrap_or(cfg.observe);
        let stop = self.stop_ov.unwrap_or(cfg.stop);
        let pre_log = self.log;
        let (u0, v0) = match self.seed_ov {
            Some(seed) => seed,
            None => (cfg.u0.clone(), cfg.v0.clone()),
        };

        // The immersed body (if any) makes this the penalized solver; else the body-free solver.
        let solver = build_solver(cfg)?;
        let mut state: QttState<R> = (quantize_2d(&u0, &cfg.trunc)?, quantize_2d(&v0, &cfg.trunc)?);

        // Drag context: active only with a body + the drag observable.
        let drag = match (&cfg.body, observe.drag) {
            (Some(b), true) => Some(DragCtx {
                mask: &b.mask,
                ubx: b.ubx,
                uby: b.uby,
                eta: b.eta,
                dx: cfg.dx,
                dy: cfg.dy,
                u_ref: b.u_ref,
                d_ref: b.d_ref,
            }),
            _ => None,
        };

        let ctx = Context {
            observe,
            projector: solver.projector(),
            drag,
            lx: cfg.lx,
            ly: cfg.ly,
            dt: cfg.dt,
        };
        let mut series = Series::new();
        ctx.sample(&state, &mut series)?;

        match stop {
            MarchStop::Fixed(n) => {
                for s in 0..n {
                    state = solver.advance(&state)?;
                    ctx.sample(&state, &mut series)?;
                    call_hook(&mut hook, &ctx, s + 1, &state);
                }
            }
            MarchStop::Steady { tol, max_steps } => {
                let mut prev_e = kinetic_energy(&state.0, &state.1)?;
                for s in 0..max_steps {
                    state = solver.advance(&state)?;
                    ctx.sample(&state, &mut series)?;
                    call_hook(&mut hook, &ctx, s + 1, &state);
                    let e = kinetic_energy(&state.0, &state.1)?;
                    if (e - prev_e).abs() < tol {
                        break;
                    }
                    prev_e = e;
                }
            }
        }

        let mut report = Report::new(cfg.name.clone());
        series.into_report(&observe, &mut report);
        let uf = dequantize_2d(&state.0, cfg.lx, cfg.ly)?;
        let vf = dequantize_2d(&state.1, cfg.lx, cfg.ly)?;
        report.set_final_field(uf.as_slice().to_vec());
        report.add_series("final_v", vf.as_slice().to_vec());
        if !pre_log.is_empty() {
            report.set_effect_log(pre_log);
        }
        Ok(report)
    }
}

/// Build a [`QttStepView`] and invoke the per-step hook, if present.
fn call_hook<R: CfdScalar + ConjugateScalar<Real = R>, H: FnMut(&QttStepView<'_, R>)>(
    hook: &mut Option<H>,
    ctx: &Context<'_, R>,
    step: usize,
    state: &QttState<R>,
) {
    if let Some(h) = hook.as_mut() {
        let view = QttStepView {
            step,
            dt: ctx.dt,
            u: &state.0,
            v: &state.1,
            lx: ctx.lx,
            ly: ctx.ly,
            projector: ctx.projector,
        };
        h(&view);
    }
}

/// A cheap, read-only view of one completed QTT step, passed to a [`QttMarchRun::run_with`] hook.
/// Exposes the step index/time, the `(u, v)` velocity trains, and the tensor-train-native
/// diagnostics computed off them.
pub struct QttStepView<'a, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    step: usize,
    dt: R,
    u: &'a CausalTensorTrain<R>,
    v: &'a CausalTensorTrain<R>,
    lx: usize,
    ly: usize,
    projector: &'a QttProjector2d<R>,
}

impl<'a, R> QttStepView<'a, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// The 1-based count of completed steps.
    pub fn step(&self) -> usize {
        self.step
    }

    /// The time step.
    pub fn dt(&self) -> R {
        self.dt
    }

    /// The elapsed time `step · dt`.
    pub fn time(&self) -> R {
        R::from_usize(self.step).expect("a step count lifts into every real field") * self.dt
    }

    /// The current `u`-velocity train.
    pub fn u(&self) -> &CausalTensorTrain<R> {
        self.u
    }

    /// The current `v`-velocity train.
    pub fn v(&self) -> &CausalTensorTrain<R> {
        self.v
    }

    /// Kinetic energy of the current state.
    ///
    /// # Errors
    /// As [`kinetic_energy`](crate::kinetic_energy).
    pub fn kinetic_energy(&self) -> Result<R, PhysicsError> {
        kinetic_energy(self.u, self.v)
    }

    /// Divergence residual of the current state.
    ///
    /// # Errors
    /// As [`divergence_residual`](crate::divergence_residual).
    pub fn divergence(&self) -> Result<R, PhysicsError> {
        divergence_residual(self.projector, self.u, self.v)
    }

    /// Maximum bond dimension across the velocity trains (the rank / compression metric).
    pub fn max_bond(&self) -> usize {
        max_bond(self.u, self.v)
    }

    /// Maximum speed of the current state.
    ///
    /// # Errors
    /// As [`max_speed`](crate::max_speed).
    pub fn max_speed(&self) -> Result<R, PhysicsError> {
        max_speed(self.u, self.v, self.lx, self.ly)
    }
}

/// The immersed-body context the drag observable contracts against (borrowed from the config).
struct DragCtx<'a, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    mask: &'a CausalTensorTrain<R>,
    ubx: R,
    uby: R,
    eta: R,
    dx: R,
    dy: R,
    u_ref: R,
    d_ref: R,
}

/// The per-step observation context — the immutable run state the sampler reads.
struct Context<'a, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    observe: QttObserve,
    projector: &'a QttProjector2d<R>,
    drag: Option<DragCtx<'a, R>>,
    lx: usize,
    ly: usize,
    dt: R,
}

impl<'a, R> Context<'a, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Sample every enabled diagnostic of `state` into the series accumulator.
    fn sample(&self, state: &QttState<R>, series: &mut Series<R>) -> Result<(), PhysicsError> {
        let (u, v) = (&state.0, &state.1);
        if self.observe.kinetic_energy {
            series.energy.push(kinetic_energy(u, v)?);
        }
        if self.observe.divergence {
            series
                .divergence
                .push(divergence_residual(self.projector, u, v)?);
        }
        if self.observe.max_speed {
            series.max_speed.push(max_speed(u, v, self.lx, self.ly)?);
        }
        if self.observe.bond {
            let b =
                R::from_usize(max_bond(u, v)).expect("a bond count lifts into every real field");
            series.bond.push(b);
        }
        if let Some(d) = &self.drag {
            let (cd, cl) = drag_lift(
                d.mask, u, v, d.ubx, d.uby, d.eta, d.dx, d.dy, d.u_ref, d.d_ref,
            )?;
            series.drag.push(cd);
            series.lift.push(cl);
        }
        Ok(())
    }
}

/// The per-step series a QTT observation accumulates.
struct Series<R: CfdScalar> {
    energy: Vec<R>,
    divergence: Vec<R>,
    max_speed: Vec<R>,
    bond: Vec<R>,
    drag: Vec<R>,
    lift: Vec<R>,
}

impl<R: CfdScalar> Series<R> {
    fn new() -> Self {
        Self {
            energy: Vec::new(),
            divergence: Vec::new(),
            max_speed: Vec::new(),
            bond: Vec::new(),
            drag: Vec::new(),
            lift: Vec::new(),
        }
    }

    fn into_report(self, observe: &QttObserve, report: &mut Report<R>) {
        if observe.kinetic_energy {
            report.add_series("kinetic_energy", self.energy);
        }
        if observe.divergence {
            report.add_series("divergence", self.divergence);
        }
        if observe.max_speed {
            report.add_series("max_speed", self.max_speed);
        }
        if observe.bond {
            report.add_series("bond", self.bond);
        }
        if !self.drag.is_empty() {
            report.add_series("drag", self.drag);
            report.add_series("lift", self.lift);
        }
    }
}
