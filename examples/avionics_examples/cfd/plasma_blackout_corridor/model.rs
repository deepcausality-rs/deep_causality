/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The corridor model. Named worlds for the flight stations, the composed coupling stack (the
//! central control loop), the example-local stages (truth/GNSS, flight loads, guidance), the
//! branch scoring, and the per-leg snapshots the gates read.
//!
//! Every tuned number, and every simplification label, lives in [`crate::constants`].

use crate::FloatType;
use crate::constants::{
    BANK_ANGLES_DEG, C_P, CAP, CDA_OVER_M, COMMS_BAND_RAD_S, DT, ETA, FlightCondition, G0,
    GNSS_VAR, GUIDANCE_GAIN, IMU_ACCEL_BIAS, IMU_GYRO_BIAS, L, L_CHAR, MAX_BANK_RAD, MAX_G_LOAD,
    MAX_HEAT_FLUX, NAV_INIT_ERR, NU, OPTICAL_VAR, P0_VAR, PEAK, PROCESS_NOISE, Q_CALIBRATION,
    REDUCED_MASS_AMU, RESIDENCE_TIME_S, RHO_REF, SMOOTH_CELLS, THETA_VIB, TRUTH_R0, TRUTH_V0,
    U_INF,
};
use deep_causality_cfd::{
    AeroForceCoupling, Ambient, BlackoutTrigger, BranchAccumulator, BranchOutcome, CfdScalar,
    CoupledField, Coupling, CyberneticCorrect, EosStage, ImuModel, InsErrorState, IonizationStage,
    MarchPause, MarchStop, NavFilter, PhysicsError, PhysicsStage, QttMarchConfig,
    QttMarchConfigBuilder, QttObserve, RecoveryTemperatureStage, ReentryNavEngine, RegimeClassify,
    Report, SafetyEnvelope, StepContext, TrajectoryNav, VibrationalLagStage, body_mask_2d,
    max_bond, quantize_2d,
};
use deep_causality_haft::LogSize;
use deep_causality_num::FromPrimitive;
use deep_causality_physics::{EARTH_GM, ks_strang_step};
use deep_causality_tensor::{CausalTensor, Truncation};

/// Lift an exact `f64` specification constant into the working precision.
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}

fn spacing() -> FloatType {
    ft(2.0 * std::f64::consts::PI) / ft((1usize << L) as f64)
}

fn trunc() -> Truncation<FloatType> {
    Truncation::<FloatType>::by_bond(CAP).expect("bond cap is valid")
}

/// The blackout trigger at the GPS L1 band.
pub fn trigger() -> BlackoutTrigger<FloatType> {
    BlackoutTrigger::new(ft(COMMS_BAND_RAD_S))
}

/// A named world: the blunt-forebody `QttMarchConfig` for one flight station.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn world(fc: &FlightCondition) -> Result<QttMarchConfig<FloatType>, PhysicsError> {
    let dx = spacing();
    let tr = trunc();
    let center = ft(std::f64::consts::PI);
    let radius = ft(fc.radius_frac * 2.0 * std::f64::consts::PI);
    let smoothing = ft(SMOOTH_CELLS) * dx;
    let mask = body_mask_2d::<FloatType>(L, L, dx, dx, center, center, radius, smoothing, &tr)?;
    let u_inf = ft(U_INF);
    QttMarchConfigBuilder::<FloatType>::new()
        .name(fc.name)
        .grid(L, L, dx, dx)
        .solver(ft(DT), ft(NU), tr)
        .seed_fn(|_, _| (u_inf, ft(0.0)))?
        .body(mask, ft(0.0), ft(0.0), ft(ETA), u_inf, dx)
        .stop(MarchStop::Fixed(fc.steps))
        .observe(
            QttObserve::default()
                .electron_density()
                .plasma_frequency()
                .heat_flux()
                .blackout_dwell(),
        )
        .build()
}

/// The candidate bank worlds: the peak station with the forebody projected by each bank angle,
/// `radius × (1 − 0.3·sin φ)`. This is how the 2-D carrier expresses banking.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn bank_worlds() -> Result<Vec<(f64, QttMarchConfig<FloatType>)>, PhysicsError> {
    let names: [&'static str; 3] = ["bank_00_deg", "bank_30_deg", "bank_60_deg"];
    BANK_ANGLES_DEG
        .iter()
        .zip(names)
        .map(|(&deg, name)| {
            let projected = PEAK.radius_frac * (1.0 - 0.3 * deg.to_radians().sin());
            let fc = FlightCondition {
                name,
                radius_frac: projected,
                ..PEAK
            };
            Ok((deg, world(&fc)?))
        })
        .collect()
}

// ── The central control loop: the corridor coupling stack (design §Stage 4) ───────────────────

/// The per-step corridor coupling, the loop body that `run_coupled` and `run_until` iterate.
/// Reads top to bottom: reacting plasma [3] (recovery temperature, the Park two-temperature
/// vibrational lag, ionization driven at the controller `Tₐ`, pressure closure), the ④ aero
/// force, the regime classifier [2], loads, truth/GNSS, navigation [4] with an IMU-sensed
/// specific force, guidance, and the cybernetic bounded-correction gate [6]. A static cons-tuple;
/// no `dyn`. An `Err`, such as an envelope breach, short-circuits the whole step.
pub fn corridor_coupling(fc: &FlightCondition) -> impl PhysicsStage<2, FloatType> {
    let imu = ImuModel::new(
        core::array::from_fn(|i| ft(IMU_ACCEL_BIAS[i])),
        core::array::from_fn(|i| ft(IMU_GYRO_BIAS[i])),
        [ft(PROCESS_NOISE); 17],
    );
    Coupling::between_steps()
        .then(RecoveryTemperatureStage::new(
            ft(fc.mach),
            ft(fc.gamma_eff),
            ft(fc.t_inf),
            ft(C_P),
        ))
        .then(VibrationalLagStage::new(
            ft(fc.t_inf),
            ft(fc.pressure_atm),
            ft(REDUCED_MASS_AMU),
            ft(THETA_VIB),
            ft(RESIDENCE_TIME_S),
        ))
        .then(
            IonizationStage::new(ft(fc.n_tot))
                .driven_by("T_a")
                .with_sheath_renewal(ft(RESIDENCE_TIME_S)),
        )
        .then(EosStage::new(ft(fc.n_tot)))
        .then(AeroForceCoupling::new(ft(RHO_REF), ft(CDA_OVER_M)))
        .then(RegimeClassify::new(ft(L_CHAR), trigger()))
        .then(FlightLoads)
        .then(TruthGnss)
        .then(
            TrajectoryNav::new([ft(PROCESS_NOISE); 17], ft(GNSS_VAR), ft(OPTICAL_VAR))
                .with_imu(imu),
        )
        .then(BankGuidance)
        .then(CyberneticCorrect::new(SafetyEnvelope::new(
            ft(MAX_HEAT_FLUX),
            ft(MAX_G_LOAD),
            ft(MAX_BANK_RAD),
        )))
        .build()
}

/// The corridor's initial coupled field: the ambient, the station's mean free path, the truth
/// vehicle state, and the navigation engine seeded with a 50-m-class initial INS error.
pub fn initial_field(fc: &FlightCondition) -> CoupledField<FloatType> {
    let mut field = CoupledField::new(Ambient::new(ft(NU), ft(U_INF), None));
    seed_station(&mut field, fc);
    field.set_scalar("truth_state", state_vec(TRUTH_R0, TRUTH_V0));
    let nav_r0: [FloatType; 3] = core::array::from_fn(|i| ft(TRUTH_R0[i] + NAV_INIT_ERR[i]));
    let nav_v0: [FloatType; 3] = core::array::from_fn(|i| ft(TRUTH_V0[i]));
    let filter = NavFilter::new(InsErrorState::<FloatType>::zero(), [ft(P0_VAR); 17]);
    field.set_nav(ReentryNavEngine::new(nav_r0, nav_v0, ft(EARTH_GM), filter));
    field
}

/// Carry a paused leg's coupled field into the next station. The clone brings the navigation
/// engine, the reacting fraction, the truth state, and the provenance log along; only the station
/// constants are re-seeded.
pub fn carry_field<S>(
    pause: &MarchPause<'_, FloatType, S>,
    fc: &FlightCondition,
) -> CoupledField<FloatType> {
    let mut field = pause.field().clone();
    seed_station(&mut field, fc);
    field
}

/// Station-dependent field constants: the mean free path the regime classifier reads.
fn seed_station(field: &mut CoupledField<FloatType>, fc: &FlightCondition) {
    field.set_scalar("mean_free_path", Vec::from([ft(fc.mean_free_path)]));
}

fn state_vec(r: [f64; 3], v: [f64; 3]) -> Vec<FloatType> {
    Vec::from([ft(r[0]), ft(r[1]), ft(r[2]), ft(v[0]), ft(v[1]), ft(v[2])])
}

// ── Example-local stages (corridor wiring, not library physics) ───────────────────────────────

/// Publishes the sensed flight loads the envelope gate reads: the carrier's Brinkman wall
/// heat-flux integral (published each step by the coupled loop, `T_w = 0` reference) rescaled to
/// W·m⁻², and the g-load off the ④ aero channel. Zero heating until the first temperature field
/// lands (the standard one-step operator split).
#[derive(Debug, Clone, Copy)]
pub struct FlightLoads;

impl PhysicsStage<2, FloatType> for FlightLoads {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let wall = field
            .scalar("wall_heat_flux")
            .and_then(|q| q.first().copied())
            .unwrap_or_else(|| ft(0.0));
        let q = ft(Q_CALIBRATION) * wall.abs();
        let a = field.aero_force().unwrap_or([ft(0.0); 3]);
        let g = norm3(a) / ft(G0);
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
        ft(amplitude * (2.0 * u - 1.0))
    })
}

/// The truth vehicle plus the GNSS constellation. Advances the true state with the true ④ aero
/// force, then publishes the position fix with receiver noise ([`fix_noise`]). The fix is always
/// broadcast; whether the receiver can use it is the corridor's denial gate, since
/// `TrajectoryNav` folds it only when the classifier says the link is up. The navigation drifts
/// anyway: its IMU senses the same force through an accelerometer bias.
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
        let kick = field.aero_force().unwrap_or([ft(0.0); 3]);
        let (r1, v1) = ks_strang_step(r, v, ft(EARTH_GM), ctx.dt(), |_r, _v| kick)?;
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

/// A deliberately aggressive proportional guidance law: desired bank proportional to sensed
/// heating. The command exceeds the envelope's bank cap, so the cybernetic gate visibly bounds
/// it, and each bounding lands in the provenance log.
#[derive(Debug, Clone, Copy)]
pub struct BankGuidance;

impl PhysicsStage<2, FloatType> for BankGuidance {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let heat = field
            .scalar("heat_flux")
            .and_then(|h| h.first().copied())
            .unwrap_or_else(|| ft(0.0));
        field.set_control_action(ft(GUIDANCE_GAIN) * heat);
        Ok(())
    }
}

// ── Snapshots + branch scoring + witnesses ────────────────────────────────────────────────────

/// The per-leg witness the gates read, taken from a paused march's carried field.
pub struct LegSnapshot {
    pub name: String,
    pub steps: usize,
    pub errored: bool,
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

/// Snapshot a paused leg: regime, plasma, loads, and the navigation error against the carried
/// truth state.
pub fn snapshot<S>(name: &str, pause: &MarchPause<'_, FloatType, S>) -> LegSnapshot {
    let field = pause.field();
    let regime = field.regime();
    let truth = field.scalar("truth_state").unwrap_or(&[]);
    let nav = field.nav();
    let nav_err_m = match (nav, truth.len() >= 3) {
        (Some(engine), true) => {
            let p = engine.position();
            let d: [FloatType; 3] = core::array::from_fn(|i| p[i] - truth[i]);
            norm3(d)
        }
        _ => ft(f64::NAN),
    };
    LegSnapshot {
        name: name.to_string(),
        steps: pause.step(),
        errored: pause.error().is_some(),
        regime_model: regime.map(|r| r.model.name()).unwrap_or("unclassified"),
        knudsen: regime.map(|r| r.knudsen).unwrap_or_else(|| ft(f64::NAN)),
        gnss_denied: regime.map(|r| r.gnss_denied).unwrap_or(false),
        ne_peak: field.scalar("n_e").map(peak).unwrap_or_else(|| ft(0.0)),
        plasma_frequency: regime
            .map(|r| r.plasma_frequency)
            .unwrap_or_else(|| ft(0.0)),
        heat_flux: scalar0(field, "heat_flux"),
        g_load: scalar0(field, "g_load"),
        nav_err_m,
        nav_var: nav
            .map(|e| e.position_variance())
            .unwrap_or_else(|| ft(0.0)),
        log_entries: field.log().len(),
    }
}

/// One scored counterfactual branch: the corridor [5] outcome plus its witnesses.
pub struct BranchScore {
    pub bank_deg: f64,
    pub outcome: BranchOutcome<FloatType>,
    pub has_alternation_marker: bool,
    pub ne_peak: FloatType,
    /// The branch's final `(u, v)` fields; the compression witness re-quantizes them.
    pub report_final: (Vec<FloatType>, Vec<FloatType>),
}

/// Score one branch report into a [`BranchOutcome`]. The per-step *sensed* heating (the wall
/// heat-flux series the loads stage publishes) and the link denial fold through the Stage-3
/// [`BranchAccumulator`]; the close is the t²-law miss over the branch's real blackout dwell,
/// `miss = ½·|b|·dwell²` with `|b|` the accelerometer-bias magnitude the INS integrates unaided.
pub fn score_branch(bank_deg: f64, report: &Report<FloatType>) -> BranchScore {
    let heat = report.series("heat_flux").unwrap_or(&[]);
    let wp = report.series("plasma_frequency").unwrap_or(&[]);
    let band = ft(COMMS_BAND_RAD_S);
    let mut acc = BranchAccumulator::new(ft(bank_deg.to_radians()));
    for (i, &q) in heat.iter().enumerate() {
        let denied = wp.get(i).is_some_and(|&w| w > band);
        acc.observe(q, denied, ft(DT));
    }
    let dwell = report
        .series("blackout_dwell")
        .and_then(|d| d.first().copied())
        .unwrap_or_else(|| ft(0.0));
    let bias: [FloatType; 3] = core::array::from_fn(|i| ft(IMU_ACCEL_BIAS[i]));
    let miss = ft(0.5) * norm3(bias) * dwell * dwell;
    let outcome = acc.finish(miss);
    BranchScore {
        bank_deg,
        outcome,
        has_alternation_marker: report
            .effect_log()
            .map(|l| format!("{l}").contains("!!ContextAlternation!!"))
            .unwrap_or(false),
        ne_peak: report.series("n_e").map(peak).unwrap_or_else(|| ft(0.0)),
        report_final: (
            report.final_field().unwrap_or(&[]).to_vec(),
            report.series("final_v").unwrap_or(&[]).to_vec(),
        ),
    }
}

/// Pick the committed branch: minimum integrated thermal load. That is this corridor's scoring
/// rule; a flight design would weight all four outcome components.
pub fn pick_committed(branches: &[BranchScore]) -> usize {
    branches
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.outcome
                .thermal_load
                .partial_cmp(&b.outcome.thermal_load)
                .expect("thermal loads are finite")
        })
        .map(|(i, _)| i)
        .expect("at least one branch")
}

/// The tensor-compression witness: re-quantize the committed branch's final fields under the
/// corridor's round policy and read the peak bond dimension against the dense grid.
pub fn compression_witness(final_fields: &(Vec<FloatType>, Vec<FloatType>)) -> (usize, usize) {
    let n = 1usize << L;
    let tr = trunc();
    let bond = CausalTensor::new(final_fields.0.clone(), vec![n, n])
        .ok()
        .zip(CausalTensor::new(final_fields.1.clone(), vec![n, n]).ok())
        .and_then(|(u, v)| Some((quantize_2d(&u, &tr).ok()?, quantize_2d(&v, &tr).ok()?)))
        .map(|(tu, tv)| max_bond(&tu, &tv))
        .unwrap_or(usize::MAX);
    (bond, n * n)
}

fn peak<R: CfdScalar>(xs: &[R]) -> R {
    xs.iter()
        .copied()
        .fold(R::zero(), |a, x| if x > a { x } else { a })
}

fn norm3(v: [FloatType; 3]) -> FloatType {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn scalar0(field: &CoupledField<FloatType>, name: &str) -> FloatType {
    field
        .scalar(name)
        .and_then(|s| s.first().copied())
        .unwrap_or_else(|| ft(0.0))
}
