/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The corridor model: the bank worlds of the counterfactual sweep, the per-leg snapshots the
//! gates read, and the branch scoring. The shared machinery (stages, constants, the world
//! builder, the coupling stack) lives in `avionics_examples::blackout`.
//!
//! Every tuned number, and every simplification label, lives in [`crate::constants`] and the
//! shared `blackout::constants`.

use crate::FloatType;
use crate::constants::{AIM_CROSS_RANGE_M, BANK_ANGLES_DEG, FINE_SPAN_STEPS, FINE_STEP_DEG, STEPS};
use avionics_examples::shared::constants::{COMMS_BAND_RAD_S, DT_FLIGHT, IMU_ACCEL_BIAS, L};
use avionics_examples::shared::{utils, world};
use deep_causality_cfd::{
    BranchAccumulator, BranchOutcome, CompressibleMarchConfig, CompressiblePause, CoupledField,
    PhysicsError, Report, max_bond, quantize_2d,
};
use deep_causality_haft::LogSize;
use deep_causality_tensor::CausalTensor;

/// A named descent world carrying its commanded bank as a world-published constant. The
/// counterfactual branches differ *only* in that command.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn descent_world(
    name: &'static str,
    bank_deg: f64,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world::descent_world(
        name,
        world::standard_atmosphere(),
        STEPS,
        &[("commanded_bank", utils::rad(bank_deg))],
    )
}

/// The candidate bank worlds of the branch study, keyed by commanded degrees.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn bank_worlds() -> Result<Vec<(f64, CompressibleMarchConfig<FloatType>)>, PhysicsError> {
    let names: [&'static str; 6] = [
        "bank_00_deg",
        "bank_05_deg",
        "bank_10_deg",
        "bank_15_deg",
        "bank_20_deg",
        "bank_40_deg",
    ];
    BANK_ANGLES_DEG
        .iter()
        .zip(names)
        .map(|(&deg, name)| Ok((deg, descent_world(name, deg)?)))
        .collect()
}

/// The fine-sweep candidate worlds: [`FINE_SPAN_STEPS`] steps up and down from the coarse
/// winner at [`FINE_STEP_DEG`] resolution, clamped into the coarse sweep's span. The coarse
/// winner itself flies again in the middle, so the refinement can only confirm or improve it.
/// Clamping at the span edges can duplicate a candidate; a duplicate branch flies the same
/// command and scores identically, which is wasteful but harmless.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn fine_bank_worlds(
    winner_deg: f64,
) -> Result<Vec<(f64, CompressibleMarchConfig<FloatType>)>, PhysicsError> {
    const NAMES: [&str; 2 * FINE_SPAN_STEPS + 1] = [
        "fine_bank_00",
        "fine_bank_01",
        "fine_bank_02",
        "fine_bank_03",
        "fine_bank_04",
        "fine_bank_05",
        "fine_bank_06",
        "fine_bank_07",
        "fine_bank_08",
        "fine_bank_09",
        "fine_bank_10",
    ];
    let lo = BANK_ANGLES_DEG[0];
    let hi = BANK_ANGLES_DEG[BANK_ANGLES_DEG.len() - 1];
    NAMES
        .iter()
        .enumerate()
        .map(|(k, name)| {
            let offset = (k as f64 - FINE_SPAN_STEPS as f64) * FINE_STEP_DEG;
            let deg = (winner_deg + offset).clamp(lo, hi);
            Ok((deg, descent_world(name, deg)?))
        })
        .collect()
}

/// Carry a paused leg's coupled field into the next diagnostic segment. The clone brings the
/// navigation engine, the truth state, the evolved projections, and the provenance log along
/// unchanged; the descent world itself never changes between legs.
pub fn carry_field<S>(pause: &CompressiblePause<'_, FloatType, S>) -> CoupledField<FloatType> {
    pause.field().clone()
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

/// One scored counterfactual branch: the corridor outcome plus its witnesses.
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

/// The aim point of the branch study: the ballistic (zero-bank) terminal state offset
/// cross-range in the direction a positive bank pushes toward, so steering is what closes it.
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

/// Score one branch report into a [`BranchOutcome`]. The per-step sensed heating and link
/// denial fold through the [`BranchAccumulator`]; the close is the **trajectory-derived** miss
/// from the branch's terminal truth state to the shared aim point. The t²-law proxy is computed
/// alongside as the printed cross-check.
pub fn score_branch(bank_deg: f64, report: &Report<FloatType>, aim: [FloatType; 3]) -> BranchScore {
    let heat = report.series("heat_flux").unwrap_or(&[]);
    let wp = report.series("plasma_frequency").unwrap_or(&[]);
    let band = utils::ft(COMMS_BAND_RAD_S);
    let mut acc = BranchAccumulator::new(utils::rad(bank_deg));
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

/// The tensor-compression witness: re-quantize the committed branch's final evolved fields
/// under the corridor's round policy and read the peak bond dimension against the dense grid.
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
