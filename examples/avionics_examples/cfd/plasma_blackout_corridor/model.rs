/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The corridor model. The descent world (one compressible world per commanded bank), the
//! composed coupling stack (the central control loop), the example-local stages (freestream
//! feeds, loads, truth/GNSS, guidance), the branch scoring, and the per-leg snapshots the gates
//! read.
//!
//! Every tuned number, and every simplification label, lives in [`crate::constants`].

use crate::constants::{
    AIM_CROSS_RANGE_M, AIR_MEAN_MOLECULAR_MASS_KG, AIR_MOLECULE_DIAMETER_M, ATMOSPHERE,
    BANK_ANGLES_DEG, CDA_OVER_M, COMMS_BAND_RAD_S, DT_FLIGHT, DT_SOLVER, FALLBACK_N_TOT,
    FALLBACK_PRESSURE_ATM, G0, GAMMA_EFF, GNSS_VAR, IMU_ACCEL_BIAS, IMU_GYRO_BIAS, L, L_CHAR,
    L_OVER_D, MAX_BANK_RAD, MAX_G_LOAD, MAX_HEAT_FLUX, N_REF, NAV_INIT_ERR, NOSE_RADIUS_M,
    OPTICAL_VAR, P0_DIAG, Q_DIAG, REDUCED_MASS_AMU, RESIDENCE_TIME_S, RHO_REF, S_REF, SEED_P_HAT,
    SEED_RHO_HAT, SEED_U_HAT, STEPS, SUTTON_GRAVES_K, T_REF, T_VE_INITIAL, THETA_VIB,
    TRUTH_ALTITUDE_0, TRUTH_V0, U_REF,
};
use crate::{FloatType, utils};
use deep_causality_cfd::{
    Ambient, AtmosphereRow, BankSteeredLift, BranchAccumulator, BranchOutcome,
    CompressibleMarchConfig, CompressibleMarchConfigBuilder, CompressiblePause, CoupledField,
    Coupling, CyberneticCorrect, DescentSchedule, ImuModel, InsErrorState, IonizationStage,
    MarchStop, NavFilter, PhysicsError, PhysicsStage, QttObserve, ReentryNavEngine,
    ReferenceScales, RegimeClassify, Report, SafetyEnvelope, StepContext, TrajectoryNav,
    VibrationalLagStage, max_bond, quantize_2d,
};
use deep_causality_haft::LogSize;
use deep_causality_physics::{EARTH_GM, EARTH_RADIUS, ks_strang_step};
use deep_causality_tensor::CausalTensor;

/// A named descent world: the same continuous-descent physics, carrying its commanded bank as a
/// world-published constant. The counterfactual branches differ *only* in that command.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn descent_world(
    name: &'static str,
    bank_deg: f64,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    let rows: Vec<AtmosphereRow<FloatType>> = ATMOSPHERE
        .iter()
        .map(|&(alt, n, t, a)| AtmosphereRow {
            altitude_m: utils::ft(alt),
            n_tot: utils::ft(n),
            temperature: utils::ft(t),
            sound_speed: utils::ft(a),
        })
        .collect();
    let schedule = DescentSchedule::new(rows, utils::ft(GAMMA_EFF))?;
    let n = 1usize << L;
    let dx = utils::ft(1.0) / utils::ft(n as f64);
    CompressibleMarchConfigBuilder::<FloatType>::new()
        .name(name)
        .grid(L, L, dx, dx)
        .solver(
            utils::ft(DT_SOLVER),
            utils::ft(S_REF),
            utils::ft(GAMMA_EFF),
            utils::trunc(),
        )
        .flight_dt(utils::ft(DT_FLIGHT))
        .seed_fn(|_, _| {
            (
                utils::ft(SEED_RHO_HAT),
                utils::ft(SEED_U_HAT),
                utils::ft(0.0),
                utils::ft(SEED_P_HAT),
            )
        })?
        .stop(MarchStop::Fixed(STEPS))
        .observe(
            QttObserve::default()
                .electron_density()
                .plasma_frequency()
                .heat_flux()
                .blackout_dwell(),
        )
        .schedule(schedule)
        .reference(ReferenceScales {
            t_ref: utils::ft(T_REF),
            n_ref: utils::ft(N_REF),
            u_ref: utils::ft(U_REF),
        })
        .publish_constant("commanded_bank", utils::ft(bank_deg.to_radians()))
        .build()
}

/// The candidate bank worlds of the branch study, keyed by commanded degrees.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn bank_worlds() -> Result<Vec<(f64, CompressibleMarchConfig<FloatType>)>, PhysicsError> {
    let names: [&'static str; 3] = ["bank_00_deg", "bank_20_deg", "bank_40_deg"];
    BANK_ANGLES_DEG
        .iter()
        .zip(names)
        .map(|(&deg, name)| Ok((deg, descent_world(name, deg)?)))
        .collect()
}

// ── The central control loop: the corridor coupling stack ─────────────────────────────────────

/// The per-step corridor coupling, the loop body every leg and branch iterates. Reads top to
/// bottom: the Park two-temperature chemistry on the **evolved** marched state (the vibrational
/// lag clocked by the evolved per-cell pressure; ionization at the controller `Tₐ` on the evolved
/// per-cell density), the freestream feeds, the regime classifier [2], the 3-DOF bank-steered ④
/// aero force, the Sutton-Graves loads, truth/GNSS, navigation [4] with an IMU-sensed specific
/// force, the commanded-bank guidance, and the cybernetic bounded-correction gate [6]. A static
/// cons-tuple; no `dyn`. An `Err`, such as an envelope breach, short-circuits the whole step.
pub fn corridor_coupling() -> impl PhysicsStage<2, FloatType> {
    let imu = ImuModel::new(
        core::array::from_fn(|i| utils::ft(IMU_ACCEL_BIAS[i])),
        core::array::from_fn(|i| utils::ft(IMU_GYRO_BIAS[i])),
        core::array::from_fn(|i| utils::ft(Q_DIAG[i])),
    );
    Coupling::between_steps()
        .then(
            VibrationalLagStage::new(
                utils::ft(T_VE_INITIAL),
                utils::ft(FALLBACK_PRESSURE_ATM),
                utils::ft(REDUCED_MASS_AMU),
                utils::ft(THETA_VIB),
                utils::ft(RESIDENCE_TIME_S),
            )
            .with_pressure_field("pressure_atm"),
        )
        .then(
            IonizationStage::new(utils::ft(FALLBACK_N_TOT))
                .with_density_field("n_tot")
                .driven_by("T_a")
                .with_sheath_renewal(utils::ft(RESIDENCE_TIME_S)),
        )
        .then(FreestreamFeeds)
        .then(RegimeClassify::new(utils::ft(L_CHAR), utils::trigger()))
        .then(
            BankSteeredLift::new(
                utils::ft(RHO_REF),
                utils::ft(CDA_OVER_M),
                utils::ft(L_OVER_D),
            )
            .with_speed_field("equivalent_airspeed"),
        )
        .then(SuttonGravesLoads)
        .then(TruthGnss)
        .then(
            TrajectoryNav::new(
                core::array::from_fn(|i| utils::ft(Q_DIAG[i])),
                utils::ft(GNSS_VAR),
                utils::ft(OPTICAL_VAR),
            )
            .with_imu(imu),
        )
        .then(CommandedBank)
        .then(CyberneticCorrect::new(SafetyEnvelope::new(
            utils::ft(MAX_HEAT_FLUX),
            utils::ft(MAX_G_LOAD),
            utils::ft(MAX_BANK_RAD),
        )))
        .build()
}

/// The corridor's initial coupled field: the truth vehicle at the descent start and the
/// navigation engine seeded with a 50-m-class initial INS error. Everything else — freestream,
/// post-shock state, chemistry inputs — is *evolved* from here; no station constants.
pub fn initial_field() -> CoupledField<FloatType> {
    let mut field = CoupledField::new(Ambient::new(utils::ft(0.01), utils::ft(0.0), None));
    let r0 = [EARTH_RADIUS + TRUTH_ALTITUDE_0, 0.0, 0.0];
    field.set_scalar("truth_state", state_vec(r0, TRUTH_V0));
    let nav_r0: [FloatType; 3] = core::array::from_fn(|i| utils::ft(r0[i] + NAV_INIT_ERR[i]));
    let nav_v0: [FloatType; 3] = core::array::from_fn(|i| utils::ft(TRUTH_V0[i]));
    let filter = NavFilter::new(
        InsErrorState::<FloatType>::zero(),
        core::array::from_fn(|i| utils::ft(P0_DIAG[i])),
    );
    field.set_nav(ReentryNavEngine::new(
        nav_r0,
        nav_v0,
        utils::ft(EARTH_GM),
        filter,
    ));
    field
}

/// Carry a paused leg's coupled field into the next diagnostic segment. The clone brings the
/// navigation engine, the truth state, the evolved projections, and the provenance log along
/// unchanged — the descent world itself never changes between legs.
pub fn carry_field<S>(pause: &CompressiblePause<'_, FloatType, S>) -> CoupledField<FloatType> {
    pause.field().clone()
}

fn state_vec(r: [f64; 3], v: [f64; 3]) -> Vec<FloatType> {
    Vec::from([
        utils::ft(r[0]),
        utils::ft(r[1]),
        utils::ft(r[2]),
        utils::ft(v[0]),
        utils::ft(v[1]),
        utils::ft(v[2]),
    ])
}

// ── Example-local stages (corridor wiring, not library physics) ───────────────────────────────

/// Derives the per-step freestream feeds from the carrier's published flight scalars:
///
/// * `"mean_free_path"` — the freestream Knudsen driver, `λ = 1/(√2·π·d²·n_∞)` from the
///   scheduled number density (hard-sphere air).
/// * `"equivalent_airspeed"` — `EAS = V·√(ρ_∞/ρ_ref)`, so the lift stage's fixed-density dynamic
///   pressure `½·ρ_ref·EAS²` equals the true `½·ρ_∞·V²` at every altitude.
///
/// A no-op until the carrier has published the flight scalars.
#[derive(Debug, Clone, Copy)]
pub struct FreestreamFeeds;

impl PhysicsStage<2, FloatType> for FreestreamFeeds {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let n_inf = utils::scalar0(field, "freestream_n");
        if n_inf <= utils::ft(0.0) {
            return Ok(());
        }
        let speed = utils::scalar0(field, "flight_speed");
        let sigma = utils::ft(core::f64::consts::SQRT_2 * core::f64::consts::PI)
            * utils::ft(AIR_MOLECULE_DIAMETER_M * AIR_MOLECULE_DIAMETER_M);
        let mfp = utils::ft(1.0) / (sigma * n_inf);
        let rho_inf = n_inf * utils::ft(AIR_MEAN_MOLECULAR_MASS_KG);
        let eas = speed * (rho_inf / utils::ft(RHO_REF)).sqrt();
        field.set_scalar("mean_free_path", Vec::from([mfp]));
        field.set_scalar("equivalent_airspeed", Vec::from([eas]));
        Ok(())
    }
}

/// Publishes the sensed flight loads the envelope gate reads: the Sutton-Graves stagnation
/// heating `q = k·√(ρ_∞/R_n)·V³` from the **evolved schedule density and flight speed**, and the
/// g-load off the ④ aero channel.
#[derive(Debug, Clone, Copy)]
pub struct SuttonGravesLoads;

impl PhysicsStage<2, FloatType> for SuttonGravesLoads {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let n_inf = utils::scalar0(field, "freestream_n");
        let speed = utils::scalar0(field, "flight_speed");
        let rho_inf = n_inf * utils::ft(AIR_MEAN_MOLECULAR_MASS_KG);
        let q = utils::ft(SUTTON_GRAVES_K)
            * (rho_inf / utils::ft(NOSE_RADIUS_M)).sqrt()
            * speed
            * speed
            * speed;
        let a = field.aero_force().unwrap_or([utils::ft(0.0); 3]);
        let g = utils::norm3(a) / utils::ft(G0);
        field.set_scalar("heat_flux", Vec::from([q]));
        field.set_scalar("g_load", Vec::from([g]));
        Ok(())
    }
}

/// Deterministic receiver noise for the published fix: a golden-ratio low-discrepancy sequence
/// per axis, scaled so the per-axis variance is exactly [`GNSS_VAR`] (uniform on `±σ√3`).
/// Reproducible on every run, with no RNG dependency, and consistent with the filter's `R`.
pub fn fix_noise(step: usize) -> [FloatType; 3] {
    const PHI: f64 = 0.618_033_988_749_894_9;
    let amplitude = (GNSS_VAR * 3.0).sqrt();
    core::array::from_fn(|axis| {
        let stride = PHI * (1.0 + 0.37 * axis as f64);
        let u = ((step as f64 + 1.0) * stride).fract();
        utils::ft(amplitude * (2.0 * u - 1.0))
    })
}

/// The truth vehicle plus the GNSS constellation. Advances the true state with the true ④ aero
/// force (drag **and** the bank-steered lift), then publishes the position fix with receiver
/// noise ([`fix_noise`]). The fix is always broadcast; whether the receiver can use it is the
/// corridor's denial gate, since `TrajectoryNav` folds it only when the classifier says the link
/// is up. The navigation drifts anyway: its IMU senses the same force through an accelerometer
/// bias.
#[derive(Debug, Clone, Copy)]
pub struct TruthGnss;

impl PhysicsStage<2, FloatType> for TruthGnss {
    fn apply(
        &self,
        ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let Some(state) = field.scalar("truth_state") else {
            return Ok(());
        };
        let r = [state[0], state[1], state[2]];
        let v = [state[3], state[4], state[5]];
        let kick = field.aero_force().unwrap_or([utils::ft(0.0); 3]);
        let (r1, v1) = ks_strang_step(r, v, utils::ft(EARTH_GM), ctx.dt(), |_r, _v| kick)?;
        field.set_scalar(
            "truth_state",
            Vec::from([r1[0], r1[1], r1[2], v1[0], v1[1], v1[2]]),
        );
        let noise = fix_noise(ctx.step());
        field.set_scalar(
            "gnss_fix",
            Vec::from([r1[0] + noise[0], r1[1] + noise[1], r1[2] + noise[2]]),
        );
        Ok(())
    }
}

/// The guidance law: command the world's published bank (`"commanded_bank"`, the constant each
/// counterfactual world carries). The raw command lands in the control channel; the cybernetic
/// gate clamps it into the envelope, and the lift stage flies the clamped value on the next step
/// (the one-step actuation lag).
#[derive(Debug, Clone, Copy)]
pub struct CommandedBank;

impl PhysicsStage<2, FloatType> for CommandedBank {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let bank = utils::scalar0(field, "commanded_bank");
        field.set_control_action(bank);
        Ok(())
    }
}

// ── Snapshots + branch scoring + witnesses ────────────────────────────────────────────────────

/// The per-leg witness the gates read, taken from a paused march's carried field.
pub struct LegSnapshot {
    pub name: String,
    pub steps: usize,
    pub errored: bool,
    pub altitude_km: FloatType,
    pub mach: FloatType,
    pub regime_model: &'static str,
    pub knudsen: FloatType,
    pub gnss_denied: bool,
    pub ne_peak: FloatType,
    pub plasma_frequency: FloatType,
    pub heat_flux: FloatType,
    pub g_load: FloatType,
    pub nav_err_m: FloatType,
    pub nav_var: FloatType,
    pub log_entries: usize,
}

/// Snapshot a paused leg: flight state, regime, plasma, loads, and the navigation error against
/// the carried truth state.
pub fn snapshot<S>(name: &str, pause: &CompressiblePause<'_, FloatType, S>) -> LegSnapshot {
    let field = pause.field();
    let regime = field.regime();
    let truth = field.scalar("truth_state").unwrap_or(&[]);
    let nav = field.nav();
    let nav_err_m = match (nav, truth.len() >= 3) {
        (Some(engine), true) => {
            let p = engine.position();
            let d: [FloatType; 3] = core::array::from_fn(|i| p[i] - truth[i]);
            utils::norm3(d)
        }
        _ => utils::ft(f64::NAN),
    };
    LegSnapshot {
        name: name.to_string(),
        steps: pause.step(),
        errored: pause.error().is_some(),
        altitude_km: utils::scalar0(field, "flight_altitude") / utils::ft(1000.0),
        mach: utils::scalar0(field, "flight_mach"),
        regime_model: regime.map(|r| r.model.name()).unwrap_or("unclassified"),
        knudsen: regime
            .map(|r| r.knudsen)
            .unwrap_or_else(|| utils::ft(f64::NAN)),
        gnss_denied: regime.map(|r| r.gnss_denied).unwrap_or(false),
        ne_peak: field
            .scalar("n_e")
            .map(utils::peak)
            .unwrap_or_else(|| utils::ft(0.0)),
        plasma_frequency: regime
            .map(|r| r.plasma_frequency)
            .unwrap_or_else(|| utils::ft(0.0)),
        heat_flux: utils::scalar0(field, "heat_flux"),
        g_load: utils::scalar0(field, "g_load"),
        nav_err_m,
        nav_var: nav
            .map(|e| e.position_variance())
            .unwrap_or_else(|| utils::ft(0.0)),
        log_entries: field.log().len(),
    }
}

/// One scored counterfactual branch: the corridor [5] outcome plus its witnesses.
pub struct BranchScore {
    pub bank_deg: f64,
    pub outcome: BranchOutcome<FloatType>,
    /// The t²-law dead-reckoning proxy, printed as a cross-check beside the trajectory-derived
    /// miss: `½·|b|·dwell²` with `|b|` the accelerometer-bias magnitude.
    pub t2_miss_m: FloatType,
    /// The branch's terminal truth position (from the report's terminal trajectory states).
    pub terminal: [FloatType; 3],
    pub has_alternation_marker: bool,
    /// The peak evolved electron density over the branch window (printed beside the outcome).
    pub ne_peak: FloatType,
    /// The branch's final evolved fields (T_tr and n_tot); the compression witness re-quantizes
    /// them.
    pub report_final: (Vec<FloatType>, Vec<FloatType>),
}

/// The aim point of the branch study: the ballistic (zero-bank) terminal state offset cross-range
/// in the direction a positive bank pushes toward, so steering is what closes it.
pub fn aim_point(ballistic_terminal: [FloatType; 3]) -> [FloatType; 3] {
    [
        ballistic_terminal[0],
        ballistic_terminal[1],
        ballistic_terminal[2] - utils::ft(AIM_CROSS_RANGE_M),
    ]
}

/// The terminal truth position on a branch report.
pub fn terminal_position(report: &Report<FloatType>) -> [FloatType; 3] {
    let truth = report.series("final_truth_state").unwrap_or(&[]);
    core::array::from_fn(|i| truth.get(i).copied().unwrap_or_else(|| utils::ft(f64::NAN)))
}

/// Score one branch report into a [`BranchOutcome`]. The per-step sensed heating and link denial
/// fold through the Stage-3 [`BranchAccumulator`]; the close is the **trajectory-derived** miss
/// from the branch's terminal truth state to the shared aim point. The t²-law proxy is computed
/// alongside as the printed cross-check.
pub fn score_branch(bank_deg: f64, report: &Report<FloatType>, aim: [FloatType; 3]) -> BranchScore {
    let heat = report.series("heat_flux").unwrap_or(&[]);
    let wp = report.series("plasma_frequency").unwrap_or(&[]);
    let band = utils::ft(COMMS_BAND_RAD_S);
    let mut acc = BranchAccumulator::new(utils::ft(bank_deg.to_radians()));
    for (i, &q) in heat.iter().enumerate() {
        let denied = wp.get(i).is_some_and(|&w| w > band);
        acc.observe(q, denied, utils::ft(DT_FLIGHT));
    }
    let dwell = report
        .series("blackout_dwell")
        .and_then(|d| d.first().copied())
        .unwrap_or_else(|| utils::ft(0.0));
    let bias: [FloatType; 3] = core::array::from_fn(|i| utils::ft(IMU_ACCEL_BIAS[i]));
    let t2_miss_m = utils::ft(0.5) * utils::norm3(bias) * dwell * dwell;
    let terminal = terminal_position(report);
    let outcome = acc.finish_at(terminal, aim);
    BranchScore {
        bank_deg,
        outcome,
        t2_miss_m,
        terminal,
        has_alternation_marker: report
            .effect_log()
            .map(|l| format!("{l}").contains("!!ContextAlternation!!"))
            .unwrap_or(false),
        ne_peak: report
            .series("n_e")
            .map(utils::peak)
            .unwrap_or_else(|| utils::ft(0.0)),
        report_final: (
            report.final_field().unwrap_or(&[]).to_vec(),
            report.series("final_n_tot").unwrap_or(&[]).to_vec(),
        ),
    }
}

/// Pick the committed branch: minimum trajectory-derived miss distance. That is this corridor's
/// scoring rule; a flight design would weight all four outcome components.
pub fn pick_committed(branches: &[BranchScore]) -> usize {
    branches
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.outcome
                .miss_distance
                .partial_cmp(&b.outcome.miss_distance)
                .expect("miss distances are finite")
        })
        .map(|(i, _)| i)
        .expect("at least one branch")
}

/// The tensor-compression witness: re-quantize the committed branch's final evolved fields under
/// the corridor's round policy and read the peak bond dimension against the dense grid.
pub fn compression_witness(final_fields: &(Vec<FloatType>, Vec<FloatType>)) -> (usize, usize) {
    let n = 1usize << L;
    let tr = utils::trunc();
    let bond = CausalTensor::new(final_fields.0.clone(), vec![n, n])
        .ok()
        .zip(CausalTensor::new(final_fields.1.clone(), vec![n, n]).ok())
        .and_then(|(t, d)| Some((quantize_2d(&t, &tr).ok()?, quantize_2d(&d, &tr).ok()?)))
        .map(|(tt, td)| max_bond(&tt, &td))
        .unwrap_or(usize::MAX);
    (bond, n * n)
}

/// Count the carrier rebuilds recorded in a provenance log rendering.
pub fn rebuild_count(rendered: &str) -> usize {
    rendered
        .lines()
        .filter(|l| l.contains("carrier rebuilt at step"))
        .count()
}
