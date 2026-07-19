/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The owned configuration container for the **compressible** coupled march (the corridor's
//! evolved-state carrier), plus the descent-schedule types that close the flow↔navigation loop.
//!
//! The carrier marches the nondimensional two-temperature-family Euler state
//! (`[ρ̂, m̂x, m̂y, Ê]`, ideal gas with `p̂ = ρ̂·T̂`) and publishes physical projections by
//! rescaling with the fixed [`ReferenceScales`]. A [`DescentSchedule`] ties the marched layer to
//! the flight: each step the truth vehicle's altitude and speed select the freestream from a
//! cited standard-atmosphere table, the exact Rankine-Hugoniot jump gives the post-shock state,
//! and the carrier enforces it on an **inflow strip** — the shock-fitted boundary of the marched
//! layer. `dt_flight` is the corridor's one compressed-time constant: the seconds of flight each
//! coupled step represents.

use crate::CfdScalar;
use crate::solvers::ForcingRegion;
use crate::types::flow_config::{MarchStop, QttObserve};
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::{EARTH_RADIUS, PhysicsError};
use deep_causality_tensor::{CausalTensor, Truncation};

/// One row of the descent atmosphere table: the freestream at one altitude.
#[derive(Debug, Clone, Copy)]
pub struct AtmosphereRow<R> {
    /// Geometric altitude, m.
    pub altitude_m: R,
    /// Freestream heavy-particle number density, m⁻³.
    pub n_tot: R,
    /// Freestream temperature, K.
    pub temperature: R,
    /// Freestream speed of sound, m·s⁻¹.
    pub sound_speed: R,
}

/// The descent schedule: a standard-atmosphere table evaluated at the truth vehicle's state each
/// step, closing the navigation→flow direction of the corridor's two-way coupling.
#[derive(Debug, Clone)]
pub struct DescentSchedule<R: CfdScalar> {
    pub(crate) table: Vec<AtmosphereRow<R>>,
    /// Effective ratio of specific heats through the shock (reacting air).
    pub(crate) gamma_eff: R,
    /// The radius the altitude is measured against (defaults to the Earth mean radius).
    pub(crate) reference_radius: R,
    /// Inflow-strip width in grid columns (the shock-fitted boundary of the marched layer).
    pub(crate) strip_cols: usize,
    /// Relative drift of the required wave speed beyond the built `s_ref` that triggers a solver
    /// rebuild (each rebuild is logged to provenance).
    pub(crate) rebuild_tol: R,
}

impl<R: CfdScalar> DescentSchedule<R> {
    /// A schedule over `table` (at least two rows, sorted by ascending altitude) with the
    /// reacting effective `gamma_eff`. Defaults: Earth mean radius, a 2-column inflow strip, and
    /// a 20% rebuild tolerance.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] on a short or unsorted table, on a
    /// `gamma_eff` that is not finite and `> 1` (the Rankine-Hugoniot jump divides by
    /// `gamma_eff − 1`), or on a row whose `n_tot`, `temperature`, or `sound_speed` is not
    /// finite and positive (each later feeds a division or a Mach/post-shock input).
    pub fn new(table: Vec<AtmosphereRow<R>>, gamma_eff: R) -> Result<Self, PhysicsError> {
        if table.len() < 2 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DescentSchedule needs at least two atmosphere rows".into(),
            ));
        }
        if table.windows(2).any(|w| w[1].altitude_m <= w[0].altitude_m) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DescentSchedule table must be sorted by ascending altitude".into(),
            ));
        }
        if !gamma_eff.is_finite() || gamma_eff <= R::one() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DescentSchedule gamma_eff must be finite and > 1".into(),
            ));
        }
        let positive = |x: R| x.is_finite() && x > R::zero();
        if table.iter().any(|row| {
            !row.altitude_m.is_finite()
                || !positive(row.n_tot)
                || !positive(row.temperature)
                || !positive(row.sound_speed)
        }) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DescentSchedule rows need a finite altitude and finite, positive n_tot, \
                 temperature, and sound_speed"
                    .into(),
            ));
        }
        let radius = R::from_f64(EARTH_RADIUS).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(EARTH_RADIUS) failed".into())
        })?;
        let tol = R::from_f64(0.2)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.2) failed".into()))?;
        Ok(Self {
            table,
            gamma_eff,
            reference_radius: radius,
            strip_cols: 2,
            rebuild_tol: tol,
        })
    }

    /// Override the altitude reference radius.
    pub fn with_reference_radius(mut self, radius: R) -> Self {
        self.reference_radius = radius;
        self
    }

    /// Override the inflow-strip width (columns).
    pub fn with_strip_cols(mut self, cols: usize) -> Self {
        self.strip_cols = cols;
        self
    }

    /// Override the rebuild tolerance (relative wave-speed drift).
    pub fn with_rebuild_tolerance(mut self, tol: R) -> Self {
        self.rebuild_tol = tol;
        self
    }

    /// The freestream at `altitude_m`, linearly interpolated and clamped to the table ends.
    pub fn sample(&self, altitude_m: R) -> AtmosphereRow<R> {
        let first = self.table[0];
        let last = self.table[self.table.len() - 1];
        if altitude_m <= first.altitude_m {
            return first;
        }
        if altitude_m >= last.altitude_m {
            return last;
        }
        let mut lo = first;
        for w in self.table.windows(2) {
            if altitude_m <= w[1].altitude_m {
                lo = w[0];
                let hi = w[1];
                let t = (altitude_m - lo.altitude_m) / (hi.altitude_m - lo.altitude_m);
                let lerp = |a: R, b: R| a + t * (b - a);
                return AtmosphereRow {
                    altitude_m,
                    n_tot: lerp(lo.n_tot, hi.n_tot),
                    temperature: lerp(lo.temperature, hi.temperature),
                    sound_speed: lerp(lo.sound_speed, hi.sound_speed),
                };
            }
        }
        lo
    }

    /// The reacting effective ratio of specific heats.
    pub fn gamma_eff(&self) -> R {
        self.gamma_eff
    }

    /// The altitude reference radius.
    pub fn reference_radius(&self) -> R {
        self.reference_radius
    }
}

/// The fixed dimensional anchors the nondimensional marched state is rescaled by when publishing
/// physical projections (`T_tr = T̂·t_ref`, `n_tot = ρ̂·n_ref`, `speed = |û|·u_ref`). Chosen once
/// per corridor (the peak-station post-shock values are the natural pick) and never varied, so
/// the marched numbers stay O(1) across the whole descent.
#[derive(Debug, Clone, Copy)]
pub struct ReferenceScales<R> {
    /// Temperature anchor, K.
    pub t_ref: R,
    /// Number-density anchor, m⁻³.
    pub n_ref: R,
    /// Speed anchor, m·s⁻¹.
    pub u_ref: R,
}

/// The owned configuration container for a compressible coupled marching case. Holds only owned
/// specs; the same config can be run repeatedly (factual + counterfactual).
pub struct CompressibleMarchConfig<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    pub(crate) name: String,
    pub(crate) lx: usize,
    pub(crate) ly: usize,
    pub(crate) dx: R,
    pub(crate) dy: R,
    /// Effective ratio of specific heats the marcher evolves with.
    pub(crate) gamma: R,
    /// The solver's (nondimensional) step.
    pub(crate) dt_solver: R,
    /// The flight seconds each coupled step represents — the corridor's compressed-time constant.
    pub(crate) dt_flight: R,
    /// Reference wave speed sizing the implicit acoustic dissipation.
    pub(crate) s_ref: R,
    pub(crate) trunc: Truncation<R>,
    /// The nondimensional conserved seed `[ρ̂, m̂x, m̂y, Ê]`.
    pub(crate) seed: [CausalTensor<R>; 4],
    pub(crate) stop: MarchStop<R>,
    pub(crate) observe: QttObserve,
    pub(crate) schedule: Option<DescentSchedule<R>>,
    pub(crate) reference: ReferenceScales<R>,
    /// World-published constants: named single-cell scalars the carrier writes into the coupled
    /// field each step. The compressible analog of the per-station field seeds — a counterfactual
    /// world carries its own commanded inputs (e.g. a candidate bank command) that the shared
    /// coupling stack reads.
    pub(crate) constants: Vec<(&'static str, R)>,
    /// An optional masked forcing region applied after each marcher step (the de-risk plume
    /// imprint seam). `None` — the default — leaves the march path exactly as it was.
    pub(crate) forcing: Option<ForcingRegion<R>>,
}

impl<R> CompressibleMarchConfig<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// The case name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The grid mode counts `(Lx, Ly)` (the grid is `2^Lx × 2^Ly`).
    pub fn modes(&self) -> (usize, usize) {
        (self.lx, self.ly)
    }

    /// The flight seconds each coupled step represents.
    pub fn dt_flight(&self) -> R {
        self.dt_flight
    }

    /// The reference scales the physical projections are rescaled by.
    pub fn reference(&self) -> ReferenceScales<R> {
        self.reference
    }

    /// The descent schedule, if the case flies one.
    pub fn schedule(&self) -> Option<&DescentSchedule<R>> {
        self.schedule.as_ref()
    }

    /// The world-published constants (name, value), written into the field each step.
    pub fn published_constants(&self) -> &[(&'static str, R)] {
        &self.constants
    }

    /// The optional masked forcing region, if this world imprints one.
    pub fn forcing(&self) -> Option<&ForcingRegion<R>> {
        self.forcing.as_ref()
    }
}

/// A fluent builder for [`CompressibleMarchConfig`].
pub struct CompressibleMarchConfigBuilder<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    name: Option<String>,
    grid: Option<(usize, usize, R, R)>,
    solver: Option<(R, R, R, Truncation<R>)>,
    dt_flight: Option<R>,
    seed: Option<[CausalTensor<R>; 4]>,
    stop: Option<MarchStop<R>>,
    observe: QttObserve,
    schedule: Option<DescentSchedule<R>>,
    reference: Option<ReferenceScales<R>>,
    constants: Vec<(&'static str, R)>,
    forcing: Option<ForcingRegion<R>>,
}

impl<R> Default for CompressibleMarchConfigBuilder<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R> CompressibleMarchConfigBuilder<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    pub fn new() -> Self {
        Self {
            name: None,
            grid: None,
            solver: None,
            dt_flight: None,
            seed: None,
            stop: None,
            observe: QttObserve::default(),
            schedule: None,
            reference: None,
            constants: Vec::new(),
            forcing: None,
        }
    }

    /// The case name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The `2^lx × 2^ly` grid with computational spacings `(dx, dy)`.
    pub fn grid(mut self, lx: usize, ly: usize, dx: R, dy: R) -> Self {
        self.grid = Some((lx, ly, dx, dy));
        self
    }

    /// The marcher parameters: solver step `dt_solver`, reference wave speed `s_ref`, effective
    /// `gamma`, and the round policy.
    pub fn solver(mut self, dt_solver: R, s_ref: R, gamma: R, trunc: Truncation<R>) -> Self {
        self.solver = Some((dt_solver, s_ref, gamma, trunc));
        self
    }

    /// The flight seconds each coupled step represents (the compressed-time constant).
    pub fn flight_dt(mut self, dt_flight: R) -> Self {
        self.dt_flight = Some(dt_flight);
        self
    }

    /// Seed the nondimensional primitives from a closure over the unit square:
    /// `(x, y) -> (ρ̂, û, v̂, p̂)`, converted to the conserved state with the configured `gamma`.
    /// Call after [`grid`](Self::grid) and [`solver`](Self::solver).
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] if `grid`/`solver` are unset; tensor errors.
    pub fn seed_fn(mut self, f: impl Fn(R, R) -> (R, R, R, R)) -> Result<Self, PhysicsError> {
        let (lx, ly, _, _) = self.grid.ok_or_else(|| {
            PhysicsError::PhysicalInvariantBroken("seed_fn requires grid(..) first".into())
        })?;
        let (_, _, gamma, _) = self.solver.ok_or_else(|| {
            PhysicsError::PhysicalInvariantBroken("seed_fn requires solver(..) first".into())
        })?;
        let (nx, ny) = (1usize << lx, 1usize << ly);
        let tot = nx * ny;
        let mut rho = Vec::with_capacity(tot);
        let mut mx = Vec::with_capacity(tot);
        let mut my = Vec::with_capacity(tot);
        let mut e = Vec::with_capacity(tot);
        let half = R::from_f64(0.5)
            .ok_or_else(|| PhysicsError::NumericalInstability("from_f64(0.5)".into()))?;
        for i in 0..nx {
            for j in 0..ny {
                let x = R::from_usize(i)
                    .and_then(|a| R::from_usize(nx).map(|b| a / b))
                    .ok_or_else(|| {
                        PhysicsError::NumericalInstability("usize lift failed".into())
                    })?;
                let y = R::from_usize(j)
                    .and_then(|a| R::from_usize(ny).map(|b| a / b))
                    .ok_or_else(|| {
                        PhysicsError::NumericalInstability("usize lift failed".into())
                    })?;
                let (d, u, v, p) = f(x, y);
                rho.push(d);
                mx.push(d * u);
                my.push(d * v);
                e.push(p / (gamma - R::one()) + half * d * (u * u + v * v));
            }
        }
        let shape = vec![nx, ny];
        self.seed = Some([
            CausalTensor::new(rho, shape.clone())?,
            CausalTensor::new(mx, shape.clone())?,
            CausalTensor::new(my, shape.clone())?,
            CausalTensor::new(e, shape)?,
        ]);
        Ok(self)
    }

    /// The march stop.
    pub fn stop(mut self, stop: MarchStop<R>) -> Self {
        self.stop = Some(stop);
        self
    }

    /// The observe opt-ins.
    pub fn observe(mut self, observe: QttObserve) -> Self {
        self.observe = observe;
        self
    }

    /// Fly a descent schedule (the truth vehicle drives the inflow through it).
    pub fn schedule(mut self, schedule: DescentSchedule<R>) -> Self {
        self.schedule = Some(schedule);
        self
    }

    /// The fixed dimensional anchors of the physical projections.
    pub fn reference(mut self, reference: ReferenceScales<R>) -> Self {
        self.reference = Some(reference);
        self
    }

    /// Publish a named single-cell constant into the coupled field each step. A counterfactual
    /// world thereby carries its own commanded inputs (e.g. a candidate bank command in
    /// `"commanded_bank"`) that the shared coupling stack reads — the compressible analog of the
    /// per-station field seeds. Repeated calls accumulate; a later call with the same name wins
    /// (it is published last).
    pub fn publish_constant(mut self, name: &'static str, value: R) -> Self {
        self.constants.push((name, value));
        self
    }

    /// Imprint a masked [`ForcingRegion`] on this world's march path: after each marcher step the
    /// conserved state is relaxed toward the region's target inside its mask (the de-risk plume
    /// seam; the analytic retro-plume enters the marched layer through this). Without it the
    /// march path is exactly the unforced marcher.
    pub fn forcing_region(mut self, region: ForcingRegion<R>) -> Self {
        self.forcing = Some(region);
        self
    }

    /// Finish the builder.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] on any missing required section.
    pub fn build(self) -> Result<CompressibleMarchConfig<R>, PhysicsError> {
        let missing =
            |what: &str| PhysicsError::PhysicalInvariantBroken(format!("builder missing {what}"));
        let (lx, ly, dx, dy) = self.grid.ok_or_else(|| missing("grid"))?;
        let (dt_solver, s_ref, gamma, trunc) = self.solver.ok_or_else(|| missing("solver"))?;
        Ok(CompressibleMarchConfig {
            name: self.name.unwrap_or_else(|| "compressible_march".into()),
            lx,
            ly,
            dx,
            dy,
            gamma,
            dt_solver,
            dt_flight: self.dt_flight.ok_or_else(|| missing("flight_dt"))?,
            s_ref,
            trunc,
            seed: self.seed.ok_or_else(|| missing("seed_fn"))?,
            stop: self.stop.ok_or_else(|| missing("stop"))?,
            observe: self.observe,
            schedule: self.schedule,
            reference: self.reference.ok_or_else(|| missing("reference"))?,
            constants: self.constants,
            forcing: self.forcing,
        })
    }
}
