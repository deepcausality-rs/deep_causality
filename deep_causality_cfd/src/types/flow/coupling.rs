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

use crate::navigation::ReentryNavEngine;
use crate::solvers::dec::diagnostics::dec_sample_velocity;
use crate::types::flow::corridor::RegimeClass;
use crate::types::{Ambient, CfdScalar};
use deep_causality_core::EffectLog;
use deep_causality_physics::{PhysicsError, SolenoidalField};
use deep_causality_topology::{LatticeComplex, Manifold};

/// The primary-state backing of a [`StepContext`]: a DEC marcher carries a metric-bearing manifold
/// and the divergence-free velocity; the QTT marcher carries neither (it publishes the projections a
/// coupling needs — e.g. a per-cell `"speed"` field — into the [`CoupledField`] instead).
enum StepBacking<'a, const D: usize, R: CfdScalar> {
    Dec {
        manifold: &'a Manifold<LatticeComplex<D, R>, R>,
        velocity: &'a SolenoidalField<R>,
    },
    Qtt,
}

/// The immutable per-step read-view a coupling stage consults: the time step and step index
/// (universal), plus a DEC-only manifold/velocity for stages that sample the primary field. The
/// backing sum type (design D8) lets the same `PhysicsStage` run under both the DEC and QTT marchers
/// with no change to the stage trait.
pub struct StepContext<'a, const D: usize, R: CfdScalar> {
    backing: StepBacking<'a, D, R>,
    dt: R,
    step: usize,
}

impl<'a, const D: usize, R: CfdScalar> StepContext<'a, D, R> {
    /// Build a DEC-backed step context (the manifold + divergence-free fluid state).
    pub fn new(
        manifold: &'a Manifold<LatticeComplex<D, R>, R>,
        velocity: &'a SolenoidalField<R>,
        dt: R,
        step: usize,
    ) -> Self {
        Self {
            backing: StepBacking::Dec { manifold, velocity },
            dt,
            step,
        }
    }

    /// Build a QTT-backed step context (no manifold/velocity — the QTT marcher publishes the
    /// primary-state projections a coupling needs as [`CoupledField`] scalars).
    pub fn qtt(dt: R, step: usize) -> Self {
        Self {
            backing: StepBacking::Qtt,
            dt,
            step,
        }
    }

    /// The metric-bearing manifold, if this is a DEC-backed context.
    pub fn manifold(&self) -> Option<&Manifold<LatticeComplex<D, R>, R>> {
        match &self.backing {
            StepBacking::Dec { manifold, .. } => Some(manifold),
            StepBacking::Qtt => None,
        }
    }

    /// The current divergence-free fluid state, if this is a DEC-backed context.
    pub fn velocity(&self) -> Option<&SolenoidalField<R>> {
        match &self.backing {
            StepBacking::Dec { velocity, .. } => Some(velocity),
            StepBacking::Qtt => None,
        }
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
    /// As [`dec_sample_velocity`]; `Err` on a QTT-backed context (no manifold to sample).
    pub fn sample_velocity(&self, point: &[R; D]) -> Result<[R; D], PhysicsError> {
        match &self.backing {
            StepBacking::Dec { manifold, velocity } => {
                dec_sample_velocity(manifold, velocity.as_one_form(), point)
            }
            StepBacking::Qtt => Err(PhysicsError::PhysicalInvariantBroken(
                "sample_velocity is unavailable on a QTT-backed StepContext".into(),
            )),
        }
    }
}

/// The owned auxiliary state threaded through the coupling between steps: named scalar fields
/// (e.g. a temperature field over cells) and the per-step [`Ambient`] a stage writes back to the
/// solver (e.g. `ν(T)`).
///
/// Two typed navigation channels ride alongside the scalar fields, carrying the ④ physics→navigation
/// coupling (design: the plasma-blackout `blackout-coupling-interface`): the **aero force** (the
/// integrated Cartesian force a trajectory stage reads as its perturbation kick) and the **control
/// action** (the bounded command a corrective stage writes, e.g. a bank-angle correction). Both are
/// `None` until a producing stage sets them, so existing couplings are unaffected.
///
/// Three further composition channels ride alongside, carrying the corridor state: the last
/// [`RegimeClass`] the classifier selected (the governing-model + GNSS-denial decision downstream
/// stages read, and against which a regime *change* is detected), the onboard
/// [`ReentryNavEngine`] a trajectory stage threads through the field (the nav *state* lives here,
/// not in the stage — stages stay immutable), and an [`EffectLog`] of provenance entries (regime
/// transitions, bounded corrections, envelope breaches) — the auditable record the flagship
/// surfaces. All start empty, so existing couplings are unaffected.
#[derive(Debug, Clone)]
pub struct CoupledField<R: CfdScalar> {
    ambient: Ambient<R>,
    scalars: Vec<(String, Vec<R>)>,
    aero_force: Option<[R; 3]>,
    control_action: Option<R>,
    regime: Option<RegimeClass<R>>,
    nav: Option<ReentryNavEngine<R>>,
    log: EffectLog,
}

impl<R: CfdScalar> CoupledField<R> {
    /// A coupled field carrying the given ambient and no scalar fields.
    pub fn new(ambient: Ambient<R>) -> Self {
        Self {
            ambient,
            scalars: Vec::new(),
            aero_force: None,
            control_action: None,
            regime: None,
            nav: None,
            log: EffectLog::new(),
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

    /// The per-step aero force (the integrated Cartesian force a trajectory stage reads as its
    /// perturbation kick), if a producing stage has set it.
    pub fn aero_force(&self) -> Option<[R; 3]> {
        self.aero_force
    }

    /// Publish the per-step aero force (a marcher / stub stage writes the ④ force channel here).
    pub fn set_aero_force(&mut self, force: [R; 3]) {
        self.aero_force = Some(force);
    }

    /// The bounded control action a corrective stage has written (e.g. a bank-angle command), if any.
    pub fn control_action(&self) -> Option<R> {
        self.control_action
    }

    /// Write the bounded control action (a corrective stage writes its clamped command here).
    pub fn set_control_action(&mut self, action: R) {
        self.control_action = Some(action);
    }

    /// The onboard navigation engine threaded through the coupling, if a nav stage has seeded it.
    pub fn nav(&self) -> Option<&ReentryNavEngine<R>> {
        self.nav.as_ref()
    }

    /// Take the navigation engine out of the field (a nav stage takes it, advances it, and puts it
    /// back — the state threads through the `CoupledField`, not through the stage).
    pub fn take_nav(&mut self) -> Option<ReentryNavEngine<R>> {
        self.nav.take()
    }

    /// Thread the navigation engine (back) into the field.
    pub fn set_nav(&mut self, nav: ReentryNavEngine<R>) {
        self.nav = Some(nav);
    }

    /// The last governing-model regime the classifier selected, if a classifier stage has run.
    pub fn regime(&self) -> Option<RegimeClass<R>> {
        self.regime
    }

    /// Record the currently-selected governing-model regime (the classifier writes it each step; a
    /// downstream stage reads it to pick its governing model).
    pub fn set_regime(&mut self, regime: RegimeClass<R>) {
        self.regime = Some(regime);
    }

    /// The provenance log accumulated by the corridor stages (regime changes, corrections, breaches).
    pub fn log(&self) -> &EffectLog {
        &self.log
    }

    /// Mutable access to the provenance log — a stage appends an audit entry here.
    pub fn log_mut(&mut self) -> &mut EffectLog {
        &mut self.log
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

/// A **stub** producer for the ④ blackout-coupling contract, standing in for the real Stage-1 marcher
/// so downstream stages (trajectory, classifier, correction) can be built and validated before it
/// lands. Each step it publishes a constant mock aero drag `[−drag, 0, 0]` into the field's aero-force
/// channel and writes a single-cell `"n_e"` scalar that is `ne_blackout` inside the scheduled step
/// window `[start, end)` and `ne_ambient` outside it — so a downstream `BlackoutTrigger` sees the
/// denial window. Swapping this stub for the real marcher stage changes no consumer.
#[derive(Debug, Clone, Copy)]
pub struct AeroBlackoutStub<R: CfdScalar> {
    drag: R,
    ne_ambient: R,
    ne_blackout: R,
    window_start: usize,
    window_end: usize,
}

impl<R: CfdScalar> AeroBlackoutStub<R> {
    /// A stub with a constant mock drag magnitude and a scheduled blackout window `[start, end)` (in
    /// step index) over which the published electron density rises from `ne_ambient` to `ne_blackout`.
    pub fn new(
        drag: R,
        ne_ambient: R,
        ne_blackout: R,
        window_start: usize,
        window_end: usize,
    ) -> Self {
        Self {
            drag,
            ne_ambient,
            ne_blackout,
            window_start,
            window_end,
        }
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for AeroBlackoutStub<R> {
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        field.set_aero_force([-self.drag, R::zero(), R::zero()]);
        let in_window = ctx.step() >= self.window_start && ctx.step() < self.window_end;
        let ne = if in_window {
            self.ne_blackout
        } else {
            self.ne_ambient
        };
        field.set_scalar("n_e", Vec::from([ne]));
        Ok(())
    }
}

/// The **real** ④ aero-force producer (Stage 1.3): the marcher→trajectory adapter that closes the
/// physics→navigation coupling with flow-derived data, replacing [`AeroBlackoutStub`]'s constant mock.
/// It reads the per-cell `"speed"` field the marcher publishes each step, forms the peak dynamic pressure
/// `q = ½·ρ_ref·U_max²`, and writes the aero *acceleration* `a = −(C_d·A/m)·q` into the aero-force channel
/// the trajectory kick reads. The electron density / blackout side of ④ is produced upstream by the
/// reacting stages ([`IonizationStage`](super::IonizationStage) writing `"n_e"`), so the real ④ producer
/// stack is `RecoveryTemperature → Ionization → AeroForceCoupling`. A no-op if `"speed"` is absent.
#[derive(Debug, Clone, Copy)]
pub struct AeroForceCoupling<R: CfdScalar> {
    speed_field: &'static str,
    rho_ref: R,
    cd_area_over_mass: R,
}

impl<R: CfdScalar> AeroForceCoupling<R> {
    /// Drive the aero acceleration from a reference freestream density `rho_ref` and the vehicle
    /// ballistic coefficient bundle `cd_area_over_mass = C_d·A/m` (so the channel carries an
    /// acceleration the trajectory adds directly).
    pub fn new(rho_ref: R, cd_area_over_mass: R) -> Self {
        Self {
            speed_field: "speed",
            rho_ref,
            cd_area_over_mass,
        }
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for AeroForceCoupling<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(speed) = field.scalar(self.speed_field) else {
            return Ok(());
        };
        let u_max = speed
            .iter()
            .copied()
            .fold(R::zero(), |a, x| if x > a { x } else { a });
        let half = R::from_f64(0.5).unwrap_or_else(R::one);
        let q = half * self.rho_ref * u_max * u_max;
        let a_drag = self.cd_area_over_mass * q;
        field.set_aero_force([R::zero() - a_drag, R::zero(), R::zero()]);
        Ok(())
    }
}
