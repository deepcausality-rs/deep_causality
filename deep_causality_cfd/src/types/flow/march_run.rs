/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **CfdFlow** DSL marching pipeline (workflow composition — the "how").
//!
//! `CfdFlow::march(&config)` injects a [`MarchConfig`] container; `.on(&manifold)` lends the
//! **caller-owned geometry** (B1); the no-arg stages resolve their sub-config from the container
//! (or apply a `*_with_config` override — ideal for counterfactuals); the terminal `run` /
//! `run_with` performs the borrow-bound materialize → seed → march → observe in one body and
//! returns an owned [`Report`] (design D2). `run_with` additionally calls a per-step hook with a
//! [`StepView`] (progress, streamed diagnostics).

use crate::solvers::DecNsConfig;
use crate::solvers::dec::diagnostics::{
    dec_divergence_residual, dec_kinetic_energy, dec_max_speed, dec_sample_velocity,
};
use crate::solvers::dec::surface_force::{
    force_coefficient, pressure_surface_force, viscous_surface_force,
};
use crate::solvers::dec::{BoundaryZone, DecNsSolver};
use crate::traits::Marcher;
use crate::types::flow::{CoupledField, PhysicsStage, Report, StepContext};
use crate::types::flow_config::march_config::MarchStop;
use crate::types::flow_config::{MarchConfig, Observe, Seed};
use crate::types::{Ambient, CfdScalar};
use deep_causality_physics::{PhysicsError, SolenoidalField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CutCellRegistry, LatticeCell, LatticeComplex, Manifold,
};

use alloc::collections::BTreeMap;

/// The injected pipeline before a geometry is bound. `.on(&manifold)` lends the caller-owned
/// geometry and yields the runnable [`MarchRun`].
pub struct MarchPipeline<
    'c,
    const D: usize,
    R: CfdScalar,
    Z: BoundaryZone<D, R>,
    C: PhysicsStage<D, R>,
> {
    config: &'c MarchConfig<D, R, Z, C>,
}

impl<'c, const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>, C: PhysicsStage<D, R>>
    MarchPipeline<'c, D, R, Z, C>
{
    /// Inject a marching-case container (called by [`CfdFlow::march`](crate::CfdFlow)).
    pub(crate) fn new(config: &'c MarchConfig<D, R, Z, C>) -> Self {
        Self { config }
    }

    /// Materialize the case's geometry internally, run, and return the report: the one-shot
    /// form for sweep bodies where each case owns a fresh grid. When several runs share one
    /// geometry (a refinement trend, a transposed solve), keep the caller-owned
    /// [`on`](Self::on) form instead, which materializes once.
    ///
    /// # Errors
    /// Any materialization, seeding, coupling, or marching failure.
    pub fn run_owned(self) -> Result<Report<R>, PhysicsError>
    where
        Z: Clone,
    {
        let manifold = self.config.materialize()?;
        self.on(&manifold).run()
    }

    /// Lend the **caller-owned geometry** the marcher borrows for the run (B1). The same config can
    /// be `.on(&other_geometry)` to transpose a proven solve to another mesh/field.
    pub fn on<'m>(
        self,
        manifold: &'m Manifold<LatticeComplex<D, R>, R>,
    ) -> MarchRun<'c, 'm, D, R, Z, C> {
        MarchRun {
            config: self.config,
            manifold,
            seed_ov: None,
            stop_ov: None,
            observe_ov: None,
            solver_ov: None,
        }
    }
}

/// A geometry-bound, runnable marching pipeline. The no-arg stages resolve their sub-config from
/// the container; the `*_with_config` variants override one sub-config (counterfactuals).
pub struct MarchRun<
    'c,
    'm,
    const D: usize,
    R: CfdScalar,
    Z: BoundaryZone<D, R>,
    C: PhysicsStage<D, R>,
> {
    config: &'c MarchConfig<D, R, Z, C>,
    manifold: &'m Manifold<LatticeComplex<D, R>, R>,
    seed_ov: Option<Seed>,
    stop_ov: Option<MarchStop<R>>,
    observe_ov: Option<Observe<D, R>>,
    solver_ov: Option<DecNsConfig<R>>,
}

impl<'c, 'm, const D: usize, R: CfdScalar, Z: BoundaryZone<D, R> + Clone, C: PhysicsStage<D, R>>
    MarchRun<'c, 'm, D, R, Z, C>
{
    /// Use the container's solver config (no-op stage, for workflow readability).
    pub fn solver(self) -> Self {
        self
    }

    /// Override the solver config for this run (counterfactual — reuse the container, swap ν/dt/…).
    pub fn solver_with_config(mut self, config: DecNsConfig<R>) -> Self {
        self.solver_ov = Some(config);
        self
    }

    /// Use the container's seed (no-op stage).
    pub fn seed(self) -> Self {
        self
    }

    /// Override the initial condition for this run.
    pub fn seed_with_config(mut self, seed: Seed) -> Self {
        self.seed_ov = Some(seed);
        self
    }

    /// Use the container's march-stop (no-op stage).
    pub fn march(self) -> Self {
        self
    }

    /// Override the march-stop for this run.
    pub fn march_with(mut self, stop: MarchStop<R>) -> Self {
        self.stop_ov = Some(stop);
        self
    }

    /// Use the container's observe set (no-op stage).
    pub fn observe(self) -> Self {
        self
    }

    /// Override the observe set for this run.
    pub fn observe_with_config(mut self, observe: Observe<D, R>) -> Self {
        self.observe_ov = Some(observe);
        self
    }

    /// Run the composed workflow, returning the owned report.
    ///
    /// # Errors
    /// Any materialization, seeding, coupling, or marching failure.
    pub fn run(self) -> Result<Report<R>, PhysicsError> {
        self.execute(None::<fn(&StepView<'_, D, R>)>)
    }

    /// Run with a per-step `hook` (called after every projected step with a [`StepView`]) — for
    /// progress lines or streamed per-step diagnostics. The series and the final report are
    /// identical to [`run`](Self::run).
    ///
    /// # Errors
    /// As [`run`](Self::run).
    pub fn run_with<H: FnMut(&StepView<'_, D, R>)>(
        self,
        hook: H,
    ) -> Result<Report<R>, PhysicsError> {
        self.execute(Some(hook))
    }

    fn execute<H: FnMut(&StepView<'_, D, R>)>(
        self,
        mut hook: Option<H>,
    ) -> Result<Report<R>, PhysicsError> {
        let config = self.config;
        let manifold = self.manifold;
        let observe = self.observe_ov.unwrap_or(config.observe);
        let seed = self.seed_ov.unwrap_or(config.seed);
        let stop = self.stop_ov.unwrap_or(config.stop);
        let solver_cfg = self.solver_ov.as_ref().unwrap_or(&config.solver);

        let registry = if observe.drag.is_some() {
            config.mesh.cut_registry()?
        } else {
            None
        };
        let ref_len = config.mesh.frontal_length();
        if observe.drag.is_some() && registry.is_none() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "CfdFlow::march: drag/lift observed but the mesh carries no immersed body".into(),
            ));
        }

        let solver = solver_cfg.materialize_with_zones(manifold, config.zones.clone())?;
        let solver = match config.moving_wall {
            Some((axis, max_side, velocity)) => {
                solver.with_moving_wall(axis, max_side, velocity)?
            }
            None => solver,
        };
        let mut state = seed.apply(&solver, manifold)?;

        // The between-step coupling carrier: the per-step ambient (ν the marcher reads) plus any
        // seeded coupled scalar fields. With the `()` coupling the ambient stays at the
        // construction ν, so the march reproduces the single-physics path bit-for-bit.
        let dt = solver.dt();
        let ncells = manifold.complex().num_cells(D);
        let mut field = CoupledField::new(Ambient::new(solver.nu(), R::zero(), None));
        for (name, value) in &config.coupled_scalars {
            field.set_scalar(name.clone(), vec![*value; ncells]);
        }

        let mut series = Series::new();
        let ctx = Context {
            observe: &observe,
            manifold,
            registry: registry.as_ref(),
            solver: &solver,
            ref_len,
        };

        ctx.sample(&state, &mut series)?;
        match stop {
            MarchStop::Fixed(n) => {
                for s in 0..n {
                    state = advance_coupled(
                        &solver,
                        manifold,
                        &config.coupling,
                        &mut field,
                        &state,
                        dt,
                        s + 1,
                    )?;
                    ctx.sample(&state, &mut series)?;
                    call_hook(&mut hook, s + 1, dt, &state, manifold);
                }
            }
            MarchStop::Steady { tol, max_steps } => {
                let mut prev_e = dec_kinetic_energy(manifold, state.as_one_form())?;
                for s in 0..max_steps {
                    state = advance_coupled(
                        &solver,
                        manifold,
                        &config.coupling,
                        &mut field,
                        &state,
                        dt,
                        s + 1,
                    )?;
                    ctx.sample(&state, &mut series)?;
                    call_hook(&mut hook, s + 1, dt, &state, manifold);
                    let e = dec_kinetic_energy(manifold, state.as_one_form())?;
                    if (e - prev_e).abs() < tol {
                        break;
                    }
                    prev_e = e;
                }
            }
        }

        let mut report = Report::new(config.name.clone());
        series.into_report(&observe, &mut report);
        if let Some(axis) = observe.centerline {
            report.add_series("centerline", centerline_profile(manifold, &state, axis)?);
        }
        report.set_final_field(state.as_one_form().as_slice().to_vec());
        Ok(report)
    }
}

/// Build a [`StepView`] and invoke the per-step hook, if present.
fn call_hook<const D: usize, R: CfdScalar, H: FnMut(&StepView<'_, D, R>)>(
    hook: &mut Option<H>,
    step: usize,
    dt: R,
    state: &SolenoidalField<R>,
    manifold: &Manifold<LatticeComplex<D, R>, R>,
) {
    if let Some(h) = hook.as_mut() {
        let view = StepView {
            step,
            dt,
            state,
            manifold,
        };
        h(&view);
    }
}

/// A cheap, read-only view of one completed step, passed to a [`MarchRun::run_with`] hook. Exposes
/// the step index/time, the raw edge cochain (for an edge-indexed probe), and convenience
/// diagnostics off the manifold.
pub struct StepView<'a, const D: usize, R: CfdScalar> {
    step: usize,
    dt: R,
    state: &'a SolenoidalField<R>,
    manifold: &'a Manifold<LatticeComplex<D, R>, R>,
}

impl<'a, const D: usize, R: CfdScalar> StepView<'a, D, R> {
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

    /// The current divergence-free state.
    pub fn state(&self) -> &SolenoidalField<R> {
        self.state
    }

    /// The raw velocity edge cochain (for an edge-indexed probe).
    pub fn one_form(&self) -> &CausalTensor<R> {
        self.state.as_one_form()
    }

    /// The metric-bearing manifold.
    pub fn manifold(&self) -> &Manifold<LatticeComplex<D, R>, R> {
        self.manifold
    }

    /// Kinetic energy of the current state.
    ///
    /// # Errors
    /// As [`dec_kinetic_energy`].
    pub fn kinetic_energy(&self) -> Result<R, PhysicsError> {
        dec_kinetic_energy(self.manifold, self.one_form())
    }

    /// Maximum pointwise speed of the current state.
    ///
    /// # Errors
    /// As [`dec_max_speed`].
    pub fn max_speed(&self) -> Result<R, PhysicsError> {
        dec_max_speed(self.manifold, self.one_form())
    }

    /// Post-projection divergence residual of the current state.
    ///
    /// # Errors
    /// As [`dec_divergence_residual`].
    pub fn divergence(&self) -> Result<R, PhysicsError> {
        dec_divergence_residual(self.manifold, self.one_form())
    }
}

/// Run the between-step coupling, then advance one projected step under the resulting ambient.
fn advance_coupled<const D: usize, R: CfdScalar, C: PhysicsStage<D, R>>(
    solver: &DecNsSolver<'_, D, R>,
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    coupling: &C,
    field: &mut CoupledField<R>,
    state: &SolenoidalField<R>,
    dt: R,
    step: usize,
) -> Result<SolenoidalField<R>, PhysicsError> {
    let ctx = StepContext::new(manifold, state, dt, step);
    coupling.apply(&ctx, field)?;
    Ok(solver.advance(state, field.ambient())?.into_state())
}

/// The per-step observation context — the immutable run state the sampler reads.
struct Context<'a, const D: usize, R: CfdScalar> {
    observe: &'a Observe<D, R>,
    manifold: &'a Manifold<LatticeComplex<D, R>, R>,
    registry: Option<&'a CutCellRegistry<D, R>>,
    solver: &'a DecNsSolver<'a, D, R>,
    ref_len: Option<R>,
}

impl<'a, const D: usize, R: CfdScalar> Context<'a, D, R> {
    /// Sample every enabled diagnostic of `state` into the series accumulator.
    fn sample(
        &self,
        state: &SolenoidalField<R>,
        series: &mut Series<R>,
    ) -> Result<(), PhysicsError> {
        let u = state.as_one_form();
        if self.observe.kinetic_energy {
            series.energy.push(dec_kinetic_energy(self.manifold, u)?);
        }
        if self.observe.divergence {
            series
                .divergence
                .push(dec_divergence_residual(self.manifold, u)?);
        }
        if self.observe.max_speed {
            series.max_speed.push(dec_max_speed(self.manifold, u)?);
        }
        if let (Some(u_ref), Some(registry), Some(ref_len)) =
            (self.observe.drag, self.registry, self.ref_len)
        {
            let (cd, cl) = surface_force_coefficients(
                self.manifold,
                registry,
                self.solver,
                state,
                u_ref,
                ref_len,
            )?;
            series.drag.push(cd);
            series.lift.push(cl);
        }
        if let Some(point) = self.observe.probe {
            let v = dec_sample_velocity(self.manifold, u, &point)?;
            series.probe.push(if D > 1 { v[1] } else { v[0] });
        }
        Ok(())
    }
}

/// The per-step series an observation accumulates.
struct Series<R: CfdScalar> {
    energy: Vec<R>,
    divergence: Vec<R>,
    max_speed: Vec<R>,
    drag: Vec<R>,
    lift: Vec<R>,
    probe: Vec<R>,
}

impl<R: CfdScalar> Series<R> {
    fn new() -> Self {
        Self {
            energy: Vec::new(),
            divergence: Vec::new(),
            max_speed: Vec::new(),
            drag: Vec::new(),
            lift: Vec::new(),
            probe: Vec::new(),
        }
    }

    fn into_report<const D: usize>(self, observe: &Observe<D, R>, report: &mut Report<R>) {
        if observe.kinetic_energy {
            report.add_series("kinetic_energy", self.energy);
        }
        if observe.divergence {
            report.add_series("divergence", self.divergence);
        }
        if observe.max_speed {
            report.add_series("max_speed", self.max_speed);
        }
        if observe.drag.is_some() {
            report.add_series("drag", self.drag);
            report.add_series("lift", self.lift);
        }
        if observe.probe.is_some() {
            report.add_series("probe", self.probe);
        }
    }
}

/// The (drag, lift) coefficients on the immersed body at the given state.
fn surface_force_coefficients<const D: usize, R: CfdScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    registry: &CutCellRegistry<D, R>,
    solver: &DecNsSolver<'_, D, R>,
    state: &SolenoidalField<R>,
    u_ref: R,
    ref_len: R,
) -> Result<(R, R), PhysicsError> {
    let (_bernoulli, static_p) = solver.pressure_diagnostic(state)?;

    let pressure: BTreeMap<[usize; D], R> = manifold
        .complex()
        .iter_cells(0)
        .zip(static_p.as_tensor().as_slice().iter())
        .map(|(vertex, &p)| (*vertex.position(), p))
        .collect();
    let cells: Vec<LatticeCell<D>> = manifold.complex().iter_cells(D).collect();
    let inv_corners = R::one()
        / R::from_usize(1usize << D).expect("the corner count lifts into every real field");

    let cell_pressure = |cell_id: usize| -> R {
        let base = *cells[cell_id].position();
        let mut sum = R::zero();
        for corner in 0..(1usize << D) {
            let mut pos = base;
            for (j, p) in pos.iter_mut().enumerate() {
                *p += (corner >> j) & 1;
            }
            if let Some(&pv) = pressure.get(&pos) {
                sum += pv;
            }
        }
        sum * inv_corners
    };

    let fp = pressure_surface_force(registry, cell_pressure);
    let fv = viscous_surface_force(manifold, registry, state.as_one_form(), solver.nu())?;
    let drag = force_coefficient(fp[0] + fv[0], u_ref, ref_len);
    let lift = if D > 1 {
        force_coefficient(fp[1] + fv[1], u_ref, ref_len)
    } else {
        R::zero()
    };
    Ok((drag, lift))
}

/// The final-state velocity profile along the domain-centered line parallel to `axis`.
fn centerline_profile<const D: usize, R: CfdScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    state: &SolenoidalField<R>,
    axis: usize,
) -> Result<Vec<R>, PhysicsError> {
    if axis >= D {
        return Err(PhysicsError::DimensionMismatch(format!(
            "centerline: axis {axis} out of range for D = {D}"
        )));
    }
    let shape = manifold.complex().shape();
    let dx = manifold
        .metric()
        .and_then(|g| g.axis_lengths())
        .ok_or_else(|| {
            PhysicsError::TopologyError(
                "centerline requires an axis-aligned geometry (per-axis spacing)".into(),
            )
        })?;
    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
    let component = (axis + 1) % D;
    let u = state.as_one_form();
    let mut profile = Vec::with_capacity(shape[axis]);
    for i in 0..shape[axis] {
        let mut point = [R::zero(); D];
        for (a, p) in point.iter_mut().enumerate() {
            *p = if a == axis {
                R::from_usize(i).expect("a lattice index lifts into every real field") * dx[a]
            } else {
                R::from_usize(shape[a]).expect("a lattice extent lifts into R") * half * dx[a]
            };
        }
        let v = dec_sample_velocity(manifold, u, &point)?;
        profile.push(v[component]);
    }
    Ok(profile)
}
