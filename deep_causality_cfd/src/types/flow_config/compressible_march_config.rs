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
    /// The rebuild budget for one leg. `None` — the default — leaves the pre-M4 behavior: the
    /// hysteresis ratchet (`1.2×` re-pin against the `(1 + tol)` gate) still bounds the *rate* of
    /// rebuilds, but no count is refused. `Some(n)` refuses the `n+1`-th, so a leg that is not
    /// converging on an acoustic envelope fails loudly instead of reporting numbers marched on a
    /// knowingly undersized one. The count is per-carrier, and a carrier is rebuilt at every leg
    /// boundary, so this is a per-leg budget; a descent-wide tally is the caller's own accounting.
    pub(crate) max_rebuilds: Option<usize>,
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
            max_rebuilds: None,
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

    /// Bound the rebuilds one leg may perform, refusing the step that would exceed it.
    ///
    /// Unbounded by default, which is the pre-M4 behavior. Set this when a leg's conditioning
    /// matters enough that repeated re-pinning should be a failure rather than a log line.
    pub fn with_rebuild_budget(mut self, max_rebuilds: usize) -> Self {
        self.max_rebuilds = Some(max_rebuilds);
        self
    }

    /// The configured rebuild budget for one leg, if any.
    pub fn rebuild_budget(&self) -> Option<usize> {
        self.max_rebuilds
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
///
/// Set through
/// [`CompressibleMarchConfigBuilder::reference`](crate::CompressibleMarchConfigBuilder::reference)
/// and read back through [`CompressibleMarchConfig::reference`].
#[derive(Debug, Clone, Copy)]
pub struct ReferenceScales<R> {
    t_ref: R,
    n_ref: R,
    u_ref: R,
}

impl<R: Copy> ReferenceScales<R> {
    /// The temperature anchor, K.
    pub fn t_ref(&self) -> R {
        self.t_ref
    }

    /// The number-density anchor, m⁻³.
    pub fn n_ref(&self) -> R {
        self.n_ref
    }

    /// The speed anchor, m·s⁻¹.
    pub fn u_ref(&self) -> R {
        self.u_ref
    }
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
    /// The optional plume re-imprint spec: refresh `forcing` from stage-published plume geometry
    /// when the commanded throttle drifts. `None` leaves the forcing region as configured.
    pub(crate) imprint: Option<PlumeImprint<R>>,
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

    /// The optional plume re-imprint spec, if this world follows a varying throttle.
    pub fn plume_imprint(&self) -> Option<&PlumeImprint<R>> {
        self.imprint.as_ref()
    }
}

/// The opt-in **plume re-imprint** spec (change `add-retropulsion-coupled-stages`, capability
/// `plume-obstruction-stage`): it lets a world's marched forcing region follow a *varying* throttle.
///
/// A `PhysicsStage` cannot reach the marched layer, so the imprint rides the carrier's existing
/// field-reading reconfiguration channel — the same `pre_step` path that already reads the
/// stage-written `"truth_state"` to set the inflow strip and rebuild the marcher. With this spec
/// present, `pre_step` reads the `"plume_max_radius"` / `"plume_penetration"` scalars a
/// `PlumeObstruction` stage published and rebuilds the forcing region from them — but only when the
/// commanded throttle drifts past [`throttle_tolerance`](Self::throttle_tolerance), logged, and
/// bounded by [`max_refreshes`](Self::max_refreshes), reusing the solver-rebuild discipline.
///
/// Without this spec the carrier's forcing region is exactly whatever
/// [`forcing_region`](CompressibleMarchConfigBuilder::forcing_region) set at world build, so the
/// march path is unchanged. The imprint is **state realism only** — the drag authority is the A0
/// correlation the `PlumeObstruction` stage applies to the force channel.
#[derive(Debug, Clone, Copy)]
pub struct PlumeImprint<R: CfdScalar> {
    pub(crate) throttle_tolerance: R,
    pub(crate) max_refreshes: usize,
    pub(crate) face_x: R,
    pub(crate) axis_y: R,
    pub(crate) smoothing_cells: R,
    pub(crate) domain_m: R,
    pub(crate) target: [R; 4],
    pub(crate) eta: R,
}

impl<R: CfdScalar> PlumeImprint<R> {
    /// A validated imprint spec.
    ///
    /// * `throttle_tolerance` — refresh only when the commanded throttle moves by more than this
    ///   (absolute).
    /// * `max_refreshes` — cap on refreshes over the march, so a noisy throttle cannot rebuild the
    ///   mask every step.
    /// * `face_x` — body-face `x̂` on the unit square; the plume hugs it and extends upstream.
    /// * `axis_y` — plume axis `ŷ` on the unit square.
    /// * `smoothing_cells` — mask smoothing skirt, in cell widths.
    /// * `domain_m` — physical width of the domain (m) the published geometry is
    ///   nondimensionalized by.
    /// * `target` — the pinned jet conserved state `[ρ̂, m̂x, m̂y, Ê]` the mask interior relaxes
    ///   toward.
    /// * `eta` — penalization strength `η` (solver time units); `η ≤ Δt` is a hard pin.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] when `throttle_tolerance`, `smoothing_cells`,
    /// `domain_m`, or `eta` is not finite and positive; when `face_x` or `axis_y` lies outside the
    /// unit square (the mask is built on it); when a target component is not finite; or when the
    /// target density is not positive (the marcher refuses a non-positive density the moment it
    /// sees one, so an imprint must not inject it).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        throttle_tolerance: R,
        max_refreshes: usize,
        face_x: R,
        axis_y: R,
        smoothing_cells: R,
        domain_m: R,
        target: [R; 4],
        eta: R,
    ) -> Result<Self, PhysicsError> {
        let positive = |x: R| x.is_finite() && x > R::zero();
        if !positive(throttle_tolerance) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "PlumeImprint: throttle_tolerance must be finite and positive".into(),
            ));
        }
        if !positive(smoothing_cells) || !positive(domain_m) || !positive(eta) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "PlumeImprint: smoothing_cells, domain_m, and eta must be finite and positive"
                    .into(),
            ));
        }
        let on_unit_square = |x: R| x.is_finite() && x >= R::zero() && x <= R::one();
        if !on_unit_square(face_x) || !on_unit_square(axis_y) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "PlumeImprint: face_x and axis_y must lie on the unit square [0, 1]".into(),
            ));
        }
        if target.iter().any(|t| !t.is_finite()) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "PlumeImprint: every target component must be finite".into(),
            ));
        }
        if !positive(target[0]) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "PlumeImprint: the target density must be finite and positive".into(),
            ));
        }
        Ok(Self {
            throttle_tolerance,
            max_refreshes,
            face_x,
            axis_y,
            smoothing_cells,
            domain_m,
            target,
            eta,
        })
    }

    /// The absolute throttle move that triggers a refresh.
    pub fn throttle_tolerance(&self) -> R {
        self.throttle_tolerance
    }

    /// The refresh cap over the march.
    pub fn max_refreshes(&self) -> usize {
        self.max_refreshes
    }

    /// The body-face `x̂` on the unit square.
    pub fn face_x(&self) -> R {
        self.face_x
    }

    /// The plume axis `ŷ` on the unit square.
    pub fn axis_y(&self) -> R {
        self.axis_y
    }

    /// The mask smoothing skirt, in cell widths.
    pub fn smoothing_cells(&self) -> R {
        self.smoothing_cells
    }

    /// The physical domain width (m) the published geometry is nondimensionalized by.
    pub fn domain_m(&self) -> R {
        self.domain_m
    }

    /// The pinned jet conserved state `[ρ̂, m̂x, m̂y, Ê]`.
    pub fn target(&self) -> [R; 4] {
        self.target
    }

    /// The penalization strength `η`.
    pub fn eta(&self) -> R {
        self.eta
    }
}

/// A fluent builder for [`CompressibleMarchConfig`]. Started by
/// [`CfdConfigBuilder::compressible_march`](crate::CfdConfigBuilder), which takes the case name.
pub struct CompressibleMarchConfigBuilder<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    name: String,
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
    imprint: Option<PlumeImprint<R>>,
}

impl<R> CompressibleMarchConfigBuilder<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// A fresh builder for the named case.
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
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
            imprint: None,
        }
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

    /// The fixed dimensional anchors of the physical projections: the temperature anchor `t_ref`
    /// (K), the number-density anchor `n_ref` (m⁻³), and the speed anchor `u_ref` (m·s⁻¹).
    pub fn reference(mut self, t_ref: R, n_ref: R, u_ref: R) -> Self {
        self.reference = Some(ReferenceScales {
            t_ref,
            n_ref,
            u_ref,
        });
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

    /// Opt into **plume re-imprint**: the carrier refreshes its forcing region from the
    /// `PlumeObstruction`-published plume geometry whenever the commanded throttle drifts past the
    /// spec's tolerance (logged, capped). Without it the forcing region stays as configured.
    pub fn plume_imprint(mut self, spec: PlumeImprint<R>) -> Self {
        self.imprint = Some(spec);
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
            name: self.name,
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
            imprint: self.imprint,
        })
    }
}
