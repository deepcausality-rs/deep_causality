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

use crate::solvers::{
    QttImmersed2d, QttIncompressible2d, divergence_residual, drag_lift, kinetic_energy, max_bond,
    max_speed,
};
use crate::tensor_bridge::{QttProjector2d, dequantize_2d, quantize_2d};
use crate::traits::Marcher;
use crate::types::CfdScalar;
use crate::types::flow::Report;
use crate::types::flow_config::{MarchStop, QttMarchConfig, QttObserve};
use alloc::vec::Vec;
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::CausalTensorTrain;

/// The `(u, v)` velocity train pair the QTT marcher carries.
type QttState<R> = (CausalTensorTrain<R>, CausalTensorTrain<R>);

/// The marcher behind the pipeline: the body-free solver, or the Brinkman-penalized immersed solver
/// when the config carries a body. Both share the `(u, v)` state and expose a projector.
enum QttSolver<R>
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
}

/// A geometry-free, runnable QTT marching pipeline. The overrides (`seed_with` / `march_with` /
/// `observe_with`) swap one spec for a counterfactual while reusing the borrowed container.
pub struct QttMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    config: &'c QttMarchConfig<R>,
    seed_ov: Option<(
        deep_causality_tensor::CausalTensor<R>,
        deep_causality_tensor::CausalTensor<R>,
    )>,
    stop_ov: Option<MarchStop<R>>,
    observe_ov: Option<QttObserve>,
}

impl<'c, R> QttMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Inject a QTT marching container (called by [`CfdFlow::qtt_march`](crate::CfdFlow)).
    pub(crate) fn new(config: &'c QttMarchConfig<R>) -> Self {
        Self {
            config,
            seed_ov: None,
            stop_ov: None,
            observe_ov: None,
        }
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

    fn execute<H: FnMut(&QttStepView<'_, R>)>(
        self,
        mut hook: Option<H>,
    ) -> Result<Report<R>, PhysicsError> {
        let cfg = self.config;
        let observe = self.observe_ov.unwrap_or(cfg.observe);
        let stop = self.stop_ov.unwrap_or(cfg.stop);
        let (u0, v0) = match self.seed_ov {
            Some(seed) => seed,
            None => (cfg.u0.clone(), cfg.v0.clone()),
        };

        // The immersed body (if any) makes this the penalized solver; else the body-free solver.
        let solver = match &cfg.body {
            Some(b) => QttSolver::Body(QttImmersed2d::new(
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
            )?),
            None => QttSolver::Free(QttIncompressible2d::new(
                cfg.lx, cfg.ly, cfg.dx, cfg.dy, cfg.dt, cfg.nu, cfg.trunc,
            )?),
        };
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
