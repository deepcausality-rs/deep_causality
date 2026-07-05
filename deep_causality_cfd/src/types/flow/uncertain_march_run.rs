/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **CfdFlow** uncertain-inflow marching pipeline (the sensor-fed causal-monad march).
//!
//! `CfdFlow::march(&config)` injects an [`UncertainMarchConfig`]; `.on(&manifold)` lends
//! the caller-owned geometry (B1); `run` / `run_with` materialize the solver, install the moving
//! wall from the zone, seed, and drive the [`inflow_march_step`] bind loop (the same kernel
//! [`march_inflow`](crate::march_inflow) packages) — surfacing an [`UncertainStepView`] per step for
//! a streamed probe — and return an owned [`Report`] (final field + the `EffectLog` dropout count).

use crate::solvers::dec::diagnostics::{
    dec_divergence_residual, dec_kinetic_energy, dec_max_speed,
};
use crate::solvers::dec::uncertain_inflow::{InflowContext, InflowMarchState, inflow_march_step};
use crate::types::CfdScalar;
use crate::types::flow::Report;
use crate::types::flow_config::UncertainMarchConfig;
use deep_causality_core::{EffectLog, EffectValue, PropagatingProcess};
use deep_causality_haft::LogSize;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{LatticeComplex, Manifold};
use deep_causality_uncertain::ProbabilisticType;

/// The injected uncertain-march pipeline before a geometry is bound.
pub struct UncertainMarchPipeline<'c, R: CfdScalar + ProbabilisticType> {
    config: &'c UncertainMarchConfig<R>,
}

impl<'c, R: CfdScalar + ProbabilisticType> UncertainMarchPipeline<'c, R> {
    pub(crate) fn new(config: &'c UncertainMarchConfig<R>) -> Self {
        Self { config }
    }

    /// Lend the caller-owned geometry the marcher borrows for the run (B1). The dimension `D` is
    /// fixed here by the manifold.
    pub fn on<'m, const D: usize>(
        self,
        manifold: &'m Manifold<LatticeComplex<D, R>, R>,
    ) -> UncertainMarchRun<'c, 'm, D, R> {
        UncertainMarchRun {
            config: self.config,
            manifold,
        }
    }
}

/// A geometry-bound, runnable uncertain-inflow march.
pub struct UncertainMarchRun<'c, 'm, const D: usize, R: CfdScalar + ProbabilisticType> {
    config: &'c UncertainMarchConfig<R>,
    manifold: &'m Manifold<LatticeComplex<D, R>, R>,
}

impl<'c, 'm, const D: usize, R: CfdScalar + ProbabilisticType> UncertainMarchRun<'c, 'm, D, R> {
    /// Run the sensor-fed march, returning the owned report.
    ///
    /// # Errors
    /// Any materialization, moving-wall, seeding, or per-step march failure.
    pub fn run(self) -> Result<Report<R>, PhysicsError> {
        self.execute(None::<fn(&UncertainStepView<'_, D, R>)>)
    }

    /// Run with a per-step `hook` called after every bind step with an [`UncertainStepView`] (for a
    /// streamed wake probe / progress). The report is identical to [`run`](Self::run).
    ///
    /// # Errors
    /// As [`run`](Self::run).
    pub fn run_with<H: FnMut(&UncertainStepView<'_, D, R>)>(
        self,
        hook: H,
    ) -> Result<Report<R>, PhysicsError> {
        self.execute(Some(hook))
    }

    fn execute<H: FnMut(&UncertainStepView<'_, D, R>)>(
        self,
        mut hook: Option<H>,
    ) -> Result<Report<R>, PhysicsError> {
        let config = self.config;
        let manifold = self.manifold;
        let zone = &config.zone;

        // Materialize the stateless solver on the lent geometry (no boundary zones — the sensor
        // drives a prescribed moving wall), then install the wall from the zone and seed.
        let solver = config.solver.materialize_with_zones(manifold, ())?;
        let mut wall = [R::zero(); D];
        if zone.flow_axis() >= D {
            return Err(PhysicsError::DimensionMismatch(format!(
                "uncertain_march: flow axis {} out of range for D = {D}",
                zone.flow_axis()
            )));
        }
        wall[zone.flow_axis()] = zone.default_inflow();
        let solver = solver.with_moving_wall(zone.wall_axis(), zone.max_side(), wall)?;
        let field = config.seed.apply(&solver, manifold)?;

        // The causal-monad march: state in the monad, the stateless solver untouched.
        let initial = InflowMarchState::new(solver, field, zone.default_inflow());
        let context = InflowContext::new(*zone, config.stream.clone());
        let mut process: PropagatingProcess<R, InflowMarchState<'m, D, R>, InflowContext<R>> =
            PropagatingProcess {
                value: EffectValue::Value(zone.default_inflow()),
                state: initial,
                context: Some(context),
                error: None,
                logs: EffectLog::new(),
            };

        for s in 0..config.steps {
            process = process.bind(inflow_march_step);
            if let Some(e) = process.error() {
                return Err(PhysicsError::PhysicalInvariantBroken(format!(
                    "uncertain_march: step {} failed: {e:?}",
                    s + 1
                )));
            }
            if let Some(h) = hook.as_mut() {
                let view = UncertainStepView {
                    step: s + 1,
                    one_form: process.state().field().as_one_form(),
                    in_dropout: process.state().in_dropout(),
                    manifold,
                };
                h(&view);
            }
        }

        let mut report = Report::new(config.name.clone());
        report.set_final_field(process.state().field().as_one_form().as_slice().to_vec());
        report.set_log_entries(process.logs().len());
        Ok(report)
    }
}

/// A read-only view of one completed uncertain-inflow step, passed to a
/// [`UncertainMarchRun::run_with`] hook. Exposes the step index, the raw edge cochain (for an
/// edge-indexed wake probe), whether this step was a sensor dropout, and convenience diagnostics.
pub struct UncertainStepView<'a, const D: usize, R: CfdScalar> {
    step: usize,
    one_form: &'a CausalTensor<R>,
    in_dropout: bool,
    manifold: &'a Manifold<LatticeComplex<D, R>, R>,
}

impl<'a, const D: usize, R: CfdScalar> UncertainStepView<'a, D, R> {
    /// The 1-based count of completed steps.
    pub fn step(&self) -> usize {
        self.step
    }

    /// The raw velocity edge cochain (for an edge-indexed probe).
    pub fn one_form(&self) -> &CausalTensor<R> {
        self.one_form
    }

    /// Whether this step was a sensor dropout (the inflow fell back to the last-good value).
    pub fn in_dropout(&self) -> bool {
        self.in_dropout
    }

    /// The metric-bearing manifold.
    pub fn manifold(&self) -> &Manifold<LatticeComplex<D, R>, R> {
        self.manifold
    }

    /// Maximum speed over the field.
    ///
    /// # Errors
    /// A DEC operator failure.
    pub fn max_speed(&self) -> Result<R, PhysicsError> {
        dec_max_speed(self.manifold, self.one_form)
    }

    /// The divergence residual (incompressibility check).
    ///
    /// # Errors
    /// A DEC operator failure.
    pub fn divergence(&self) -> Result<R, PhysicsError> {
        dec_divergence_residual(self.manifold, self.one_form)
    }

    /// The kinetic energy.
    ///
    /// # Errors
    /// A DEC operator failure.
    pub fn kinetic_energy(&self) -> Result<R, PhysicsError> {
        dec_kinetic_energy(self.manifold, self.one_form)
    }
}
