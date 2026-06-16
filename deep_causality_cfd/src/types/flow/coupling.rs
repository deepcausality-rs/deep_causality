/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `.couple` multi-physics seam (design D4): a statically-composed, between-step physics
//! pipeline run once per timestep *around* the CFD step.
//!
//! A [`PhysicsStage`] is one between-step physics transform; stages compose by the same cons-tuple
//! discipline as the boundary zones — [`PhysicsStage`] is implemented for `()` (the identity) and
//! for `(Head, Tail)` (run the head, then the tail), so a [`Coupling`] built with `.then(...)`
//! nests into a fully static pipeline with **no `dyn`**. Each stage reads the per-step
//! [`StepContext`] (the manifold, the current fluid state, `dt`) and mutates the owned
//! [`CoupledField`] — the auxiliary state (named scalar fields plus the per-step [`Ambient`] the
//! marcher reads). A stage that returns `Err` short-circuits the chain (errors propagate across the
//! whole holistic coupling via `?`). Adding a coupled physics is a small `PhysicsStage` impl, not a
//! change to the DSL core.

use crate::solvers::dec::diagnostics::dec_sample_velocity;
use crate::types::{Ambient, CfdScalar};
use deep_causality_physics::{PhysicsError, SolenoidalField};
use deep_causality_topology::{LatticeComplex, Manifold};

/// The immutable per-step context a coupling stage reads: the manifold, the current fluid state
/// (for advection / wall sampling), the time step, and the step index.
pub struct StepContext<'a, const D: usize, R: CfdScalar> {
    manifold: &'a Manifold<LatticeComplex<D, R>, R>,
    velocity: &'a SolenoidalField<R>,
    dt: R,
    step: usize,
}

impl<'a, const D: usize, R: CfdScalar> StepContext<'a, D, R> {
    /// Build a step context (called by the marcher between steps).
    pub fn new(
        manifold: &'a Manifold<LatticeComplex<D, R>, R>,
        velocity: &'a SolenoidalField<R>,
        dt: R,
        step: usize,
    ) -> Self {
        Self {
            manifold,
            velocity,
            dt,
            step,
        }
    }

    /// The metric-bearing manifold.
    pub fn manifold(&self) -> &Manifold<LatticeComplex<D, R>, R> {
        self.manifold
    }

    /// The current divergence-free fluid state.
    pub fn velocity(&self) -> &SolenoidalField<R> {
        self.velocity
    }

    /// The time step.
    pub fn dt(&self) -> R {
        self.dt
    }

    /// The current step index (0 at the seed).
    pub fn step(&self) -> usize {
        self.step
    }

    /// The fluid velocity vector at a physical point (in spacing units) — for an advecting stage.
    ///
    /// # Errors
    /// As [`dec_sample_velocity`].
    pub fn sample_velocity(&self, point: &[R; D]) -> Result<[R; D], PhysicsError> {
        dec_sample_velocity(self.manifold, self.velocity.as_one_form(), point)
    }
}

/// The owned auxiliary state threaded through the coupling between steps: named scalar fields
/// (e.g. a temperature field over cells) and the per-step [`Ambient`] a stage writes back to the
/// solver (e.g. `ν(T)`).
#[derive(Debug, Clone)]
pub struct CoupledField<R: CfdScalar> {
    ambient: Ambient<R>,
    scalars: Vec<(String, Vec<R>)>,
}

impl<R: CfdScalar> CoupledField<R> {
    /// A coupled field carrying the given ambient and no scalar fields.
    pub fn new(ambient: Ambient<R>) -> Self {
        Self {
            ambient,
            scalars: Vec::new(),
        }
    }

    /// The per-step ambient (read by the marcher after the coupling runs).
    pub fn ambient(&self) -> &Ambient<R> {
        &self.ambient
    }

    /// Mutable access to the ambient — a stage drives `ν`, the freestream, or the body force here.
    pub fn ambient_mut(&mut self) -> &mut Ambient<R> {
        &mut self.ambient
    }

    /// Insert or replace a named scalar field.
    pub fn set_scalar(&mut self, name: impl Into<String>, data: Vec<R>) {
        let name = name.into();
        if let Some(slot) = self.scalars.iter_mut().find(|(n, _)| *n == name) {
            slot.1 = data;
        } else {
            self.scalars.push((name, data));
        }
    }

    /// A named scalar field, if present.
    pub fn scalar(&self, name: &str) -> Option<&[R]> {
        self.scalars
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, d)| d.as_slice())
    }

    /// Mutable access to a named scalar field, if present.
    pub fn scalar_mut(&mut self, name: &str) -> Option<&mut Vec<R>> {
        self.scalars
            .iter_mut()
            .find(|(n, _)| n == name)
            .map(|(_, d)| d)
    }
}

/// One between-step physics transform. Implemented for `()` (identity) and `(Head, Tail)`
/// (sequential composition) so couplings compose statically; a concrete physics is a small impl.
pub trait PhysicsStage<const D: usize, R: CfdScalar> {
    /// Apply this stage's physics for one step, mutating the coupled field.
    ///
    /// # Errors
    /// Any physics failure; an `Err` short-circuits the rest of the coupling.
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError>;
}

/// The identity coupling — the empty pipeline.
impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for () {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        _field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        Ok(())
    }
}

/// Sequential composition: run `Head`, then `Tail`. `Head`'s error short-circuits `Tail`.
impl<const D: usize, R: CfdScalar, A, B> PhysicsStage<D, R> for (A, B)
where
    A: PhysicsStage<D, R>,
    B: PhysicsStage<D, R>,
{
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        self.0.apply(ctx, field)?;
        self.1.apply(ctx, field)
    }
}

/// A fluent builder for a between-step coupling — a static cons-tuple of [`PhysicsStage`]s.
pub struct Coupling<S> {
    stages: S,
}

impl Coupling<()> {
    /// Begin an empty between-step coupling.
    pub fn between_steps() -> Self {
        Coupling { stages: () }
    }
}

impl<S> Coupling<S> {
    /// Append a stage, transitioning the coupling type (the prior pipeline runs, then `stage`).
    pub fn then<T>(self, stage: T) -> Coupling<(S, T)> {
        Coupling {
            stages: (self.stages, stage),
        }
    }

    /// Finish the builder, yielding the composed stage tuple.
    pub fn build(self) -> S {
        self.stages
    }
}

/// A `Coupling` is itself a `PhysicsStage` (it delegates to its composed tuple).
impl<const D: usize, R: CfdScalar, S: PhysicsStage<D, R>> PhysicsStage<D, R> for Coupling<S> {
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        self.stages.apply(ctx, field)
    }
}

// ---------------------------------------------------------------------------
// Concrete stages
// ---------------------------------------------------------------------------

/// A first-order conduction relaxation of a named scalar field toward a wall temperature:
/// `T ← T + rate·dt·(T_wall − T)` per cell. A stand-in for `Solid::conduction()` — enough to
/// drive the `ν(T)` feedback through [`ViscosityArrhenius`]. A no-op if the field is absent.
#[derive(Debug, Clone, Copy)]
pub struct ThermalRelax<R: CfdScalar> {
    field: &'static str,
    rate: R,
    wall_temperature: R,
}

impl<R: CfdScalar> ThermalRelax<R> {
    /// Relax the `"temperature"` field toward `wall_temperature` at the given `rate`.
    pub fn new(rate: R, wall_temperature: R) -> Self {
        Self {
            field: "temperature",
            rate,
            wall_temperature,
        }
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for ThermalRelax<R> {
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let factor = self.rate * ctx.dt();
        let wall = self.wall_temperature;
        if let Some(temp) = field.scalar_mut(self.field) {
            for t in temp.iter_mut() {
                *t += factor * (wall - *t);
            }
        }
        Ok(())
    }
}

/// A temperature-dependent viscosity closure (Arrhenius form) that writes `ν(T)` into the
/// ambient — the stage that closes the thermo → fluid loop. `ν(T) = ν_ref · exp(β·(T_ref/T − 1))`,
/// so `ν = ν_ref` at `T = T_ref`. Reads the mean of the `"temperature"` field (the wall-driven
/// bulk temperature); with no temperature field it leaves `ν` unchanged.
#[derive(Debug, Clone, Copy)]
pub struct ViscosityArrhenius<R: CfdScalar> {
    field: &'static str,
    nu_ref: R,
    t_ref: R,
    beta: R,
}

impl<R: CfdScalar> ViscosityArrhenius<R> {
    /// `ν_ref` at reference temperature `t_ref`, with sensitivity `beta`.
    pub fn new(nu_ref: R, t_ref: R, beta: R) -> Self {
        Self {
            field: "temperature",
            nu_ref,
            t_ref,
            beta,
        }
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for ViscosityArrhenius<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(temp) = field.scalar(self.field) else {
            return Ok(());
        };
        if temp.is_empty() || self.t_ref <= R::zero() {
            return Ok(());
        }
        let n = R::from_usize(temp.len()).expect("the cell count lifts into every real field");
        let mean = temp.iter().fold(R::zero(), |acc, &t| acc + t) / n;
        if mean <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "ViscosityArrhenius: mean temperature must be positive".into(),
            ));
        }
        // ν(T) = ν_ref · exp(β·(T_ref/T − 1)).
        let nu = self.nu_ref * (self.beta * (self.t_ref / mean - R::one())).exp();
        field.ambient_mut().set_nu(nu);
        Ok(())
    }
}
