/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The corridor model. Named worlds for the flight stations, the composed coupling stack (the
//! central control loop), the example-local stages (truth/GNSS, flight loads, guidance), the
//! branch scoring, and the per-leg snapshots the gates read.
//!
//! Every tuned number, and every Tier-A simplification label, lives in [`crate::constants`].

use crate::FloatType;
use crate::constants::{
    BANK_ANGLES_DEG, C_P, CAP, CDA_OVER_M, COMMS_BAND_RAD_S, DRAG_MODEL_ERROR, DT, ETA,
    FlightCondition, G0, GAMMA, GNSS_VAR, GUIDANCE_GAIN, HEAT_COEFF, L, L_CHAR, MAX_BANK_RAD,
    MAX_G_LOAD, MAX_HEAT_FLUX, NAV_INIT_ERR, NU, OPTICAL_VAR, P0_VAR, PEAK, PROCESS_NOISE, RHO_REF,
    SMOOTH_CELLS, TRUTH_R0, TRUTH_V0, U_INF,
};
use deep_causality_cfd::{
    AeroForceCoupling, Ambient, BlackoutTrigger, BranchAccumulator, BranchOutcome, CfdScalar,
    CoupledField, Coupling, CyberneticCorrect, EosStage, InsErrorState, IonizationStage,
    MarchPause, MarchStop, NavFilter, PhysicsError, PhysicsStage, QttMarchConfig,
    QttMarchConfigBuilder, QttObserve, RecoveryTemperatureStage, ReentryNavEngine, RegimeClassify,
    Report, SafetyEnvelope, StepContext, TrajectoryNav, body_mask_2d, max_bond, quantize_2d,
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
                .max_speed()
                .blackout_dwell(),
        )
        .build()
}

/// The candidate bank worlds: the peak station with the forebody projected by each bank angle,
/// `radius × (1 − 0.3·sin φ)`. This is the Tier-A expression of banking.
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
/// Reads top to bottom: reacting plasma [3], the ④ aero force, the regime classifier [2], loads,
/// truth/GNSS, navigation [4], guidance, and the cybernetic bounded-correction gate [6]. A static
/// cons-tuple; no `dyn`. An `Err`, such as an envelope breach, short-circuits the whole step.
pub fn corridor_coupling(fc: &FlightCondition) -> impl PhysicsStage<2, FloatType> {
    Coupling::between_steps()
        .then(RecoveryTemperatureStage::new(
            ft(fc.mach),
            ft(GAMMA),
            ft(fc.t_inf),
            ft(C_P),
        ))
        .then(IonizationStage::new(ft(fc.n_tot)))
        .then(EosStage::new(ft(fc.n_tot)))
        .then(AeroForceCoupling::new(ft(RHO_REF), ft(CDA_OVER_M)))
        .then(RegimeClassify::new(ft(L_CHAR), trigger()))
        .then(FlightLoads)
        .then(TruthGnss)
        .then(TrajectoryNav::new(
            [ft(PROCESS_NOISE); 17],
            ft(GNSS_VAR),
            ft(OPTICAL_VAR),
        ))
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

/// Publishes the sensed flight loads the envelope gate reads: the Sutton-Graves-form stagnation
/// heating `q = K·u³` off the carrier's peak speed, and the g-load off the ④ aero channel.
#[derive(Debug, Clone, Copy)]
pub struct FlightLoads;

impl PhysicsStage<2, FloatType> for FlightLoads {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let u_peak = field.scalar("speed").map(peak).unwrap_or_else(|| ft(U_INF));
        let q = ft(HEAT_COEFF) * u_peak * u_peak * u_peak;
        let a = field.aero_force().unwrap_or([ft(0.0); 3]);
        let g = norm3(a) / ft(G0);
        field.set_scalar("heat_flux", Vec::from([q]));
        field.set_scalar("g_load", Vec::from([g]));
        Ok(())
    }
}

/// The truth vehicle plus the GNSS constellation. Advances the true state with the ④ aero force
/// *plus the 5% drag the navigation model does not know about*, then publishes the position fix.
/// The fix is always broadcast; whether the receiver can use it is the corridor's denial gate,
/// since `TrajectoryNav` folds it only when the classifier says the link is up.
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
        let aero = field.aero_force().unwrap_or([ft(0.0); 3]);
        let scale = ft(1.0 + DRAG_MODEL_ERROR);
        let kick: [FloatType; 3] = core::array::from_fn(|i| aero[i] * scale);
        let (r1, v1) = ks_strang_step(r, v, ft(EARTH_GM), ctx.dt(), |_r, _v| kick)?;
        field.set_scalar(
            "truth_state",
            Vec::from([r1[0], r1[1], r1[2], v1[0], v1[1], v1[2]]),
        );
        field.set_scalar("gnss_fix", Vec::from([r1[0], r1[1], r1[2]]));
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

/// Score one branch report into a [`BranchOutcome`]. The per-step Sutton-Graves heating and link
/// denial fold through the Stage-3 [`BranchAccumulator`]; the close is the t²-law miss over the
/// branch's real blackout dwell, `miss = ½·a_err·dwell²` with `a_err` the unmodeled 5% drag.
pub fn score_branch(bank_deg: f64, report: &Report<FloatType>) -> BranchScore {
    let speed = report.series("max_speed").unwrap_or(&[]);
    let wp = report.series("plasma_frequency").unwrap_or(&[]);
    let band = ft(COMMS_BAND_RAD_S);
    let mut acc = BranchAccumulator::new(ft(bank_deg.to_radians()));
    for (i, &u) in speed.iter().enumerate() {
        let q = ft(HEAT_COEFF) * u * u * u;
        let denied = wp.get(i).is_some_and(|&w| w > band);
        acc.observe(q, denied, ft(DT));
    }
    let dwell = report
        .series("blackout_dwell")
        .and_then(|d| d.first().copied())
        .unwrap_or_else(|| ft(0.0));
    let a_err = ft(DRAG_MODEL_ERROR * CDA_OVER_M);
    let miss = ft(0.5) * a_err * dwell * dwell;
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
