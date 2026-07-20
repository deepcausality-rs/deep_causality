/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **CfdFlow** host for the compressible carrier: the corridor's evolved-state marcher in the
//! same coupled loop, pause/fork machinery, and alternation vocabulary as the QTT host.
//!
//! `CfdFlow::march(&config)` borrows a [`CompressibleMarchConfig`] and yields a
//! runnable [`CompressibleMarchRun`]. The carrier marches the nondimensional conserved state
//! `[ρ̂, m̂x, m̂y, Ê]` with the 2-D compressible marcher and publishes **evolved** physical
//! projections each step — `"T_tr"` and `"pressure_atm"` from the equation of state, `"n_tot"`
//! from the density, `"speed"` from the momentum — so the corridor consumes marched post-shock
//! quantities instead of a reconstruction.
//!
//! With a [`DescentSchedule`] the run flies a continuous descent: each step the truth vehicle's
//! altitude and speed (read from the carried `"truth_state"`) select the freestream from the
//! atmosphere table, the exact Rankine-Hugoniot jump (`FittedNormalShock`) gives the post-shock
//! state, and the carrier enforces it on the **inflow strip** — the shock-fitted boundary of the
//! marched layer. Navigation feeds flow, flow feeds navigation. The solver is rebuilt only when
//! the required wave speed drifts past the schedule's tolerance; each rebuild lands in the
//! provenance log.

use super::carrier::{
    CarrierFork, CarrierPause, CoupledCarrier, CoupledLoopSpec, run_coupled_driver,
    run_until_driver,
};
use super::coupling::{CoupledField, PhysicsStage};
use crate::CfdScalar;
use crate::coordinate::CartesianIdentity;
use crate::solvers::{CompressibleMarcher2d, EulerStateTt2d, FittedNormalShock, ForcingRegion};
use crate::tensor_bridge::{dequantize_2d, plume_mask_2d, quantize_2d};
use crate::traits::Marcher;
use crate::types::flow::{BlackoutTrigger, Report};
use crate::types::flow_config::PlumeImprint;
use crate::types::flow_config::{
    CompressibleMarchConfig, DescentSchedule, MarchStop, QttObserve, ReferenceScales,
};
use alloc::vec::Vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_core::{AlternatableContext, AlternatableState, AlternatableValue, EffectLog};
use deep_causality_haft::{LogAddEntry, LogAppend};
use deep_causality_physics::{BOLTZMANN_CONSTANT, PhysicsError};
use deep_causality_tensor::{CausalTensor, Truncation};

/// A compressible coupled march paused mid-flight (the shared branch state of a counterfactual
/// study). Produced by [`CompressibleMarchRun::run_until`].
pub type CompressiblePause<'c, R, S> = CarrierPause<'c, R, S, CompressibleCarrier<R>, 2>;

/// One counterfactual branch forked from a [`CompressiblePause`].
pub type CompressibleFork<'p, 'c, R, S> = CarrierFork<'p, 'c, R, S, CompressibleCarrier<R>, 2>;

/// The evolved-state carrier: the 2-D compressible marcher behind the [`CoupledCarrier`] seam,
/// with the descent schedule and the shock-fitted inflow strip.
pub struct CompressibleCarrier<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    marcher: CompressibleMarcher2d<R, CartesianIdentity<R>>,
    lx: usize,
    ly: usize,
    dx: R,
    dy: R,
    gamma: R,
    dt_solver: R,
    dt_flight: R,
    s_ref: R,
    trunc: Truncation<R>,
    reference: ReferenceScales<R>,
    schedule: Option<DescentSchedule<R>>,
    /// World-published constants written into the field each step (a counterfactual world's
    /// commanded inputs, e.g. `"commanded_bank"`).
    constants: Vec<(&'static str, R)>,
    /// The world's optional masked forcing region, applied after each marcher step (the de-risk
    /// plume imprint seam). `None` leaves the march path exactly as it was.
    forcing: Option<ForcingRegion<R>>,
    /// The current nondimensional conserved inflow `[ρ̂, m̂x, m̂y, Ê]` the strip enforces.
    inflow: Option<[R; 4]>,
    /// Solver rebuilds performed while following the schedule. The `1.2x` re-pin against the
    /// `(1 + rebuild_tol)` gate is a hysteresis ratchet bounding the rate; the schedule    /// Solver rebuilds performed while following the schedule (logged; a re-pin gate caps it).apos;s optional
    /// `max_rebuilds` bounds the count. Exposed through `CoupledCarrier::rebuilds`.
    rebuilds: usize,
    /// The optional plume re-imprint spec: refresh `forcing` from stage-published plume geometry
    /// when the commanded throttle drifts. `None` leaves `forcing` exactly as configured.
    imprint: Option<PlumeImprint<R>>,
    /// The throttle the current forcing region was imprinted at (the drift reference).
    imprinted_throttle: Option<R>,
    /// Plume re-imprints performed (logged; the spec's `max_refreshes` caps it).
    imprints: usize,
}

impl<R> CompressibleCarrier<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn lift(x: f64) -> Result<R, PhysicsError> {
        R::from_f64(x).ok_or_else(|| PhysicsError::NumericalInstability("f64 lift failed".into()))
    }

    /// Decode the four conserved components to dense fields.
    fn decode(&self, state: &EulerStateTt2d<R>) -> Result<[Vec<R>; 4], PhysicsError> {
        Ok([
            dequantize_2d(&state[0], self.lx, self.ly)?
                .as_slice()
                .to_vec(),
            dequantize_2d(&state[1], self.lx, self.ly)?
                .as_slice()
                .to_vec(),
            dequantize_2d(&state[2], self.lx, self.ly)?
                .as_slice()
                .to_vec(),
            dequantize_2d(&state[3], self.lx, self.ly)?
                .as_slice()
                .to_vec(),
        ])
    }

    /// Encode four dense conserved components.
    fn encode(&self, dense: &[Vec<R>; 4]) -> Result<EulerStateTt2d<R>, PhysicsError> {
        let (nx, ny) = (1usize << self.lx, 1usize << self.ly);
        let enc = |v: &Vec<R>| -> Result<_, PhysicsError> {
            let ct = CausalTensor::new(v.clone(), alloc::vec![nx, ny])?;
            quantize_2d(&ct, &self.trunc)
        };
        Ok([
            enc(&dense[0])?,
            enc(&dense[1])?,
            enc(&dense[2])?,
            enc(&dense[3])?,
        ])
    }

    /// Refresh the masked forcing region from the plume geometry a `PlumeObstruction` stage
    /// published, so a marched imprint follows a **varying** throttle.
    ///
    /// A `PhysicsStage` cannot reach the marched layer, so the imprint rides this — the carrier's
    /// existing field-reading reconfiguration channel, the same `pre_step` path that already reads
    /// the stage-written `"truth_state"` to set the inflow strip and rebuild the marcher. The
    /// refresh reuses the solver-rebuild discipline: it fires only when the commanded throttle
    /// drifts past the spec's tolerance, it is logged, and the spec's `max_refreshes` caps it so a
    /// noisy throttle cannot rebuild the mask every step. Absent the spec this never runs and the
    /// forcing region stays exactly as configured at world build.
    ///
    /// The imprint is **state realism only** — the drag authority is the A0 correlation the
    /// `PlumeObstruction` stage applies to the force channel, never this mask.
    fn refresh_plume_imprint(
        &mut self,
        field: &mut CoupledField<R>,
        step: usize,
    ) -> Result<(), PhysicsError> {
        let Some(spec) = self.imprint else {
            return Ok(());
        };
        if self.imprints >= spec.max_refreshes {
            return Ok(());
        }
        let throttle = field
            .scalar("commanded_throttle")
            .and_then(|s| s.first().copied())
            .unwrap_or_else(R::zero);
        let drifted = match self.imprinted_throttle {
            None => throttle > R::zero(),
            Some(prev) => (throttle - prev).abs() > spec.throttle_tolerance,
        };
        if !drifted {
            return Ok(());
        }
        // The geometry the stage published last coupling (metres) → the unit square.
        let (Some(r_max_m), Some(pen_m)) = (
            field
                .scalar("plume_max_radius")
                .and_then(|s| s.first().copied()),
            field
                .scalar("plume_penetration")
                .and_then(|s| s.first().copied()),
        ) else {
            return Ok(());
        };
        let two = Self::lift(2.0)?;
        let max_radius = r_max_m / spec.domain_m;
        let half_length = (pen_m / spec.domain_m) / two;
        // The plume hugs the body face and extends upstream.
        let cx = spec.face_x - half_length;
        let mask = plume_mask_2d(
            self.lx,
            self.ly,
            self.dx,
            self.dy,
            cx,
            spec.axis_y,
            half_length,
            max_radius,
            spec.smoothing_cells * self.dx,
            &self.trunc,
        )?;
        self.forcing = Some(ForcingRegion::new(mask, spec.target, spec.eta)?);
        self.imprinted_throttle = Some(throttle);
        self.imprints += 1;
        field.log_mut().add_entry(&alloc::format!(
            "plume re-imprint at step {step}: throttle {}, R_max {} m, L_pen {} m (imprint {})",
            throttle,
            r_max_m,
            pen_m,
            self.imprints,
        ));
        Ok(())
    }

    /// Enforce the shock-fitted inflow strip (Dirichlet over the first `strip_cols` columns).
    fn enforce_inflow(&self, state: &EulerStateTt2d<R>) -> Result<EulerStateTt2d<R>, PhysicsError> {
        let (Some(inflow), Some(schedule)) = (self.inflow, self.schedule.as_ref()) else {
            return Ok(state.clone());
        };
        let ny = 1usize << self.ly;
        let mut dense = self.decode(state)?;
        for (k, component) in dense.iter_mut().enumerate() {
            for i in 0..schedule.strip_cols.min(1usize << self.lx) {
                for j in 0..ny {
                    component[i * ny + j] = inflow[k];
                }
            }
        }
        self.encode(&dense)
    }
}

impl<R> CoupledCarrier<2, R> for CompressibleCarrier<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    type Config = CompressibleMarchConfig<R>;
    type State = EulerStateTt2d<R>;
    type Seed = [CausalTensor<R>; 4];

    fn build(cfg: &CompressibleMarchConfig<R>) -> Result<Self, PhysicsError> {
        let metric = CartesianIdentity::new(cfg.lx, cfg.ly, cfg.dx, cfg.dy, cfg.trunc)?;
        let marcher =
            CompressibleMarcher2d::new(metric, cfg.gamma, cfg.dt_solver, cfg.s_ref, cfg.trunc)?;
        // A forcing mask must live on this world's quantized grid (lx + ly cores), or every
        // Hadamard against the state would fail one step in.
        if let Some(region) = &cfg.forcing {
            let want = cfg.lx + cfg.ly;
            let got = region.mask().cores().len();
            if got != want {
                return Err(PhysicsError::DimensionMismatch(alloc::format!(
                    "forcing-region mask has {got} cores but the grid quantizes to {want}"
                )));
            }
        }
        Ok(Self {
            marcher,
            lx: cfg.lx,
            ly: cfg.ly,
            dx: cfg.dx,
            dy: cfg.dy,
            gamma: cfg.gamma,
            dt_solver: cfg.dt_solver,
            dt_flight: cfg.dt_flight,
            s_ref: cfg.s_ref,
            trunc: cfg.trunc,
            reference: cfg.reference,
            schedule: cfg.schedule.clone(),
            constants: cfg.constants.clone(),
            forcing: cfg.forcing.clone(),
            inflow: None,
            rebuilds: 0,
            imprint: cfg.imprint,
            imprinted_throttle: None,
            imprints: 0,
        })
    }

    fn seed_state(&self, cfg: &CompressibleMarchConfig<R>) -> Result<Self::State, PhysicsError> {
        Ok([
            quantize_2d(&cfg.seed[0], &self.trunc)?,
            quantize_2d(&cfg.seed[1], &self.trunc)?,
            quantize_2d(&cfg.seed[2], &self.trunc)?,
            quantize_2d(&cfg.seed[3], &self.trunc)?,
        ])
    }

    fn encode_seed(&self, seed: &Self::Seed) -> Result<Self::State, PhysicsError> {
        Ok([
            quantize_2d(&seed[0], &self.trunc)?,
            quantize_2d(&seed[1], &self.trunc)?,
            quantize_2d(&seed[2], &self.trunc)?,
            quantize_2d(&seed[3], &self.trunc)?,
        ])
    }

    fn rebuilds(&self) -> usize {
        self.rebuilds
    }

    fn dt(&self) -> R {
        self.dt_flight
    }

    /// Follow the descent: sample the atmosphere at the truth vehicle's altitude, take the exact
    /// Rankine-Hugoniot post-shock state at its Mach number, nondimensionalize it into the inflow
    /// strip, publish the flight scalars, and rebuild the solver if the required wave speed
    /// drifts past the tolerance (logged).
    fn pre_step(&mut self, field: &mut CoupledField<R>, step: usize) -> Result<(), PhysicsError> {
        // The world's published constants land first, so a counterfactual world's commanded
        // inputs (e.g. a candidate bank) are on the field before any stage reads it.
        for &(name, value) in &self.constants {
            field.set_scalar(name, Vec::from([value]));
        }
        // The plume imprint follows the (now-published) throttle through this same channel.
        self.refresh_plume_imprint(field, step)?;
        let Some(schedule) = self.schedule.as_ref() else {
            return Ok(());
        };
        let Some(truth) = field.scalar("truth_state") else {
            return Ok(());
        };
        if truth.len() < 6 {
            return Ok(());
        }
        let r = (truth[0] * truth[0] + truth[1] * truth[1] + truth[2] * truth[2]).sqrt();
        let speed = (truth[3] * truth[3] + truth[4] * truth[4] + truth[5] * truth[5]).sqrt();
        let altitude = r - schedule.reference_radius;
        let row = schedule.sample(altitude);
        let mach = speed / row.sound_speed;

        // The exact RH jump when a shock stands; the freestream itself below Mach ~1.
        let shock_floor = Self::lift(1.05)?;
        let (t_post, n_post, u_post) = if mach > shock_floor {
            let shock = FittedNormalShock::new(schedule.gamma_eff)?;
            let post = shock.post_shock(row.temperature, row.n_tot, mach)?;
            (post.t2, post.n_tot2, speed * post.u_ratio)
        } else {
            (row.temperature, row.n_tot, speed)
        };

        // Nondimensionalize with the fixed reference anchors (p̂ = ρ̂·T̂ by construction).
        let rho_hat = n_post / self.reference.n_ref;
        let t_hat = t_post / self.reference.t_ref;
        let u_hat = u_post / self.reference.u_ref;
        let half = Self::lift(0.5)?;
        let p_hat = rho_hat * t_hat;
        let e_hat = p_hat / (self.gamma - R::one()) + half * rho_hat * u_hat * u_hat;
        self.inflow = Some([rho_hat, rho_hat * u_hat, R::zero(), e_hat]);

        // Flight scalars for downstream stages (loads, lift, guidance).
        field.set_scalar("flight_altitude", Vec::from([altitude]));
        field.set_scalar("flight_speed", Vec::from([speed]));
        field.set_scalar("flight_mach", Vec::from([mach]));
        field.set_scalar("freestream_n", Vec::from([row.n_tot]));

        // Rebuild when the inflow's wave speed outgrows the built acoustic envelope.
        //
        // The trigger is **one-sided and wave-speed-keyed**, and the re-pin to `1.2·s_needed`
        // against a `(1 + tol)` gate makes it a hysteresis ratchet: each successive rebuild needs
        // roughly `1.44×` further growth at the default tolerance. The nondimensional **density
        // never enters** `s_needed`, so a configuration whose density anchor is wrong is never
        // corrected here; and because a leg builds a fresh carrier, the envelope resets to the
        // world's configured value at every boundary and the trigger re-arms at that baseline.
        let s_needed = u_hat + (self.gamma * t_hat).sqrt();
        if s_needed > self.s_ref * (R::one() + schedule.rebuild_tol) {
            // The ratchet bounds the *rate* of rebuilds but states no budget. A leg that needs more
            // than the configured count is not converging on an envelope, and its numbers should
            // not be reported as results — so the bound refuses rather than silently declining to
            // rebuild (which would march on knowingly undersized acoustics).
            if let Some(max) = schedule.max_rebuilds
                && self.rebuilds >= max
            {
                field.log_mut().add_entry(&alloc::format!(
                    "carrier rebuild budget exhausted at step {step}: {} rebuilds at the cap {max}",
                    self.rebuilds,
                ));
                return Err(PhysicsError::PhysicalInvariantBroken(alloc::format!(
                    "compressible carrier: rebuild budget of {max} exhausted at step {step}"
                )));
            }
            let s_new = s_needed * Self::lift(1.2)?;
            let metric = CartesianIdentity::new(self.lx, self.ly, self.dx, self.dy, self.trunc)?;
            self.marcher =
                CompressibleMarcher2d::new(metric, self.gamma, self.dt_solver, s_new, self.trunc)?;
            self.rebuilds += 1;
            field.log_mut().add_entry(&alloc::format!(
                "carrier rebuilt at step {step}: s_ref {} -> {} (rebuild {})",
                self.s_ref,
                s_new,
                self.rebuilds,
            ));
            self.s_ref = s_new;
        }
        Ok(())
    }

    fn advance(&self, state: &Self::State) -> Result<Self::State, PhysicsError> {
        let held = self.enforce_inflow(state)?;
        let next = self.marcher.advance(&held, &())?;
        // The optional forcing region relaxes the stepped state toward its target inside the
        // mask; with no region configured this arm is never taken and the path is unchanged.
        match &self.forcing {
            None => Ok(next),
            Some(region) => region.apply(&next, self.dt_solver, &self.trunc),
        }
    }

    /// Publish the evolved physical projections: `"speed"` (m/s), `"T_tr"` (K) and
    /// `"pressure_atm"` from the equation of state, and `"n_tot"` (m⁻³) from the density. The
    /// carried scalar is relaxed in place by the coupling on this carrier (no train transport),
    /// so `kappa` is unused.
    fn publish_and_transport(
        &self,
        state: &Self::State,
        field: &mut CoupledField<R>,
        _kappa: R,
    ) -> Result<(), PhysicsError> {
        let dense = self.decode(state)?;
        let n = dense[0].len();
        let mut speed = Vec::with_capacity(n);
        let mut t_tr = Vec::with_capacity(n);
        let mut n_tot = Vec::with_capacity(n);
        let mut p_atm = Vec::with_capacity(n);
        let half = Self::lift(0.5)?;
        let kb = Self::lift(BOLTZMANN_CONSTANT)?;
        let atm = Self::lift(101_325.0)?;
        let tiny = Self::lift(1.0e-12)?;
        for (((&rho, &mx), &my), &e) in dense[0].iter().zip(&dense[1]).zip(&dense[2]).zip(&dense[3])
        {
            if rho <= R::zero() || !rho.is_finite() {
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "compressible carrier: density must stay positive".into(),
                ));
            }
            let u2 = (mx * mx + my * my) / (rho * rho);
            let p_hat = (self.gamma - R::one()) * (e - half * rho * u2);
            let p_hat = if p_hat > tiny { p_hat } else { tiny };
            let t_hat = p_hat / rho;
            let t_phys = t_hat * self.reference.t_ref;
            let n_phys = rho * self.reference.n_ref;
            speed.push(u2.sqrt() * self.reference.u_ref);
            t_tr.push(t_phys);
            n_tot.push(n_phys);
            p_atm.push(n_phys * kb * t_phys / atm);
        }
        field.set_scalar("speed", speed);
        field.set_scalar("T_tr", t_tr);
        field.set_scalar("n_tot", n_tot);
        field.set_scalar("pressure_atm", p_atm);
        Ok(())
    }

    /// Final fields: the evolved translational temperature as the primary field, plus the final
    /// density and speed series.
    /// The largest bond dimension across the four conserved tensor trains — the rank this state
    /// actually reached, which sits at or below the configured truncation cap.
    fn peak_bond(&self, state: &Self::State) -> Option<usize> {
        state.iter().map(|t| t.max_bond()).max()
    }

    fn finish(&self, state: &Self::State, report: &mut Report<R>) -> Result<(), PhysicsError> {
        let dense = self.decode(state)?;
        let n = dense[0].len();
        let mut t_tr = Vec::with_capacity(n);
        let mut n_tot = Vec::with_capacity(n);
        let mut speed = Vec::with_capacity(n);
        let half = Self::lift(0.5)?;
        let tiny = Self::lift(1.0e-12)?;
        for (((&rho, &mx), &my), &e) in dense[0].iter().zip(&dense[1]).zip(&dense[2]).zip(&dense[3])
        {
            let u2 = (mx * mx + my * my) / (rho * rho);
            let p_hat = (self.gamma - R::one()) * (e - half * rho * u2);
            let p_hat = if p_hat > tiny { p_hat } else { tiny };
            t_tr.push((p_hat / rho) * self.reference.t_ref);
            n_tot.push(rho * self.reference.n_ref);
            speed.push(u2.sqrt() * self.reference.u_ref);
        }
        report.set_final_field(t_tr);
        report.add_series("final_n_tot", n_tot);
        report.add_series("final_speed", speed);
        Ok(())
    }

    fn config_name(cfg: &CompressibleMarchConfig<R>) -> &str {
        &cfg.name
    }

    fn config_observe(cfg: &CompressibleMarchConfig<R>) -> QttObserve {
        cfg.observe
    }
}

/// A runnable compressible marching pipeline: the same config→run split, coupled loop, and
/// counterfactual vocabulary as the QTT host, over the evolved-state carrier.
pub struct CompressibleMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    config: &'c CompressibleMarchConfig<R>,
    context_ov: Option<&'c CompressibleMarchConfig<R>>,
    seed_ov: Option<[CausalTensor<R>; 4]>,
    stop_ov: Option<MarchStop<R>>,
    observe_ov: Option<QttObserve>,
    log: EffectLog,
    /// The audit-log file this run flushes to (the `save_log` verb), as a path string. `None` is
    /// the default: no sink, in-memory log only. Stored as a string so the field stays present in
    /// `no_std` builds; the `save_log` verb and the disk sink are `std`-only.
    sink_path: Option<alloc::string::String>,
}

/// `!!ContextAlternation!!` — swap the whole world (a different borrowed config) before marching.
impl<'c, R> AlternatableContext<&'c CompressibleMarchConfig<R>> for CompressibleMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn alternate_context(mut self, new_context: &'c CompressibleMarchConfig<R>) -> Self {
        self.log.add_entry(&alloc::format!(
            "!!ContextAlternation!!: world '{}' replaced with '{}'",
            self.effective_config().name(),
            new_context.name()
        ));
        self.context_ov = Some(new_context);
        self
    }
}

/// `!!StateAlternation!!` — swap the marching state (the conserved seed) before marching.
impl<'c, R> AlternatableState<[CausalTensor<R>; 4]> for CompressibleMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn alternate_state(mut self, new_state: [CausalTensor<R>; 4]) -> Self {
        self.log
            .add_entry("!!StateAlternation!!: marching seed replaced");
        self.seed_ov = Some(new_state);
        self
    }
}

/// `!!ValueAlternation!!` — inject a primary-state snapshot (the `intervene` analog); pre-run it
/// lands on the seed.
impl<'c, R> AlternatableValue<[CausalTensor<R>; 4]> for CompressibleMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn alternate_value(mut self, new_value: [CausalTensor<R>; 4]) -> Self {
        self.log
            .add_entry("!!ValueAlternation!!: primary-state snapshot injected");
        self.seed_ov = Some(new_value);
        self
    }
}

impl<'c, R> CompressibleMarchRun<'c, R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Inject a compressible marching container (called by
    /// [`CfdFlow::march`](crate::CfdFlow::march)).
    pub(crate) fn new(config: &'c CompressibleMarchConfig<R>) -> Self {
        Self {
            config,
            context_ov: None,
            seed_ov: None,
            stop_ov: None,
            observe_ov: None,
            log: EffectLog::new(),
            sink_path: None,
        }
    }

    /// Attach a disk audit-log sink at `path` (the trajectory-level `save_log` verb). Every
    /// provenance entry is flushed to the file the moment it is recorded; without it the run is
    /// unchanged (in-memory log, console rendering). Set before `.couple(..)` so the sink rides
    /// into the coupled march.
    #[cfg(feature = "std")]
    #[must_use]
    pub fn save_log(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.sink_path = Some(path.as_ref().to_string_lossy().into_owned());
        self
    }

    /// The world this run marches in: the alternated context if one was swapped in, else the
    /// injected config.
    pub(crate) fn effective_config(&self) -> &'c CompressibleMarchConfig<R> {
        self.context_ov.unwrap_or(self.config)
    }

    /// Override the march-stop for this run.
    pub fn march_with(mut self, stop: MarchStop<R>) -> Self {
        self.stop_ov = Some(stop);
        self
    }

    /// Attach the multiphysics coupling stack and open the named-stage march builder: the
    /// readable form of [`run_until`](Self::run_until) / [`run_coupled`](Self::run_coupled),
    /// reading `.couple(stack).from(state).until(event)` instead of a positional argument list.
    pub fn couple<S>(self, coupling: S) -> crate::types::flow::CoupledMarch<'c, R, S>
    where
        S: PhysicsStage<2, R>,
    {
        crate::types::flow::CoupledMarch::new(self, coupling)
    }

    /// Override the observe set for this run.
    pub fn observe_with(mut self, observe: QttObserve) -> Self {
        self.observe_ov = Some(observe);
        self
    }

    /// Run the coupled march to completion (the loop body is the composed [`PhysicsStage`]
    /// stack), returning the owned report with the evolved final fields and the provenance log.
    ///
    /// # Errors
    /// Any assembly, marching, coupling, or reporting failure.
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

        let mut carrier = CompressibleCarrier::build(cfg)?;
        let state = match self.seed_ov {
            Some(seed) => carrier.encode_seed(&seed)?,
            None => carrier.seed_state(cfg)?,
        };

        let mut field = initial;
        let mut pre_log = self.log;
        field.log_mut().append(&mut pre_log);

        let steps = match stop {
            MarchStop::Fixed(n) => n,
            MarchStop::Steady { max_steps, .. } => max_steps,
        };

        let spec = CoupledLoopSpec {
            coupling,
            trigger,
            kappa: scalar_kappa,
            steps,
        };
        // With a `save_log` sink attached, the driver flushes each step's provenance to disk; the
        // default `NoAudit` is a zero-cost no-op that keeps an unaudited run byte-for-byte unchanged.
        #[cfg(feature = "std")]
        if let Some(path) = self.sink_path {
            let mut sink = crate::types::flow::LogSink::create(path)?;
            return run_coupled_driver(&mut carrier, cfg, spec, field, state, &observe, &mut sink);
        }
        run_coupled_driver(
            &mut carrier,
            cfg,
            spec,
            field,
            state,
            &observe,
            &mut crate::types::flow::carrier::NoAudit,
        )
    }

    /// March the coupled loop **until a predicate pauses it** (or the horizon is exhausted),
    /// yielding a resumable [`CompressiblePause`]. Step failures are captured into the pause's
    /// error channel; assembly failures fail fast.
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
    ) -> Result<CompressiblePause<'c, R, S>, PhysicsError>
    where
        S: PhysicsStage<2, R>,
        P: Fn(&CoupledField<R>, usize) -> bool,
    {
        let cfg = self.effective_config();
        let stop = self.stop_ov.unwrap_or(cfg.stop);

        let carrier = CompressibleCarrier::build(cfg)?;
        let state = match self.seed_ov {
            Some(seed) => carrier.encode_seed(&seed)?,
            None => carrier.seed_state(cfg)?,
        };

        let mut field = initial;
        let mut pre_log = self.log;
        field.log_mut().append(&mut pre_log);

        let steps = match stop {
            MarchStop::Fixed(n) => n,
            MarchStop::Steady { max_steps, .. } => max_steps,
        };

        let spec = CoupledLoopSpec {
            coupling,
            trigger,
            kappa: scalar_kappa,
            steps,
        };
        #[cfg(feature = "std")]
        if let Some(path) = self.sink_path {
            let mut sink = crate::types::flow::LogSink::create(path)?;
            return run_until_driver(carrier, cfg, spec, field, predicate, state, &mut sink);
        }
        run_until_driver(
            carrier,
            cfg,
            spec,
            field,
            predicate,
            state,
            &mut crate::types::flow::carrier::NoAudit,
        )
    }
}
