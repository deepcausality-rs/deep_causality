/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The corridor model: the bank commands of the two-round counterfactual study, the branch
//! scoring the reduction reads, the per-leg snapshots, and the two gating sequences — the campaign
//! sequence over the branch rows and the trajectory sequence over the flown legs. The shared
//! machinery (stages, the world builder, the coupling stack) lives in `avionics_examples::shared`.
//!
//! Every tuned number, and every simplification label, lives in [`crate::constants`] and the
//! shared `blackout::constants`.

use crate::FloatType;
use crate::constants::{
    AIM_CROSS_RANGE_M, BANK_ANGLES_DEG, DIVERGENCE_MIN_M, EXIT_ALTITUDE_BAND_KM, FINE_SPAN_STEPS,
    FINE_STEP_DEG, MAX_REBUILDS, MISS_IMPROVEMENT_FACTOR, RAMC_EXIT_WINDOW_KM, STEPS,
    WALL_CLOCK_BUDGET_S,
};
use avionics_examples::shared::constants::{
    CAP, COMMS_BAND_RAD_S, DT_FLIGHT, IMU_ACCEL_BIAS, L, RAMC_NE_REFERENCE,
};
use avionics_examples::shared::utils::{ft, norm3};
use avionics_examples::shared::{utils, world};
use deep_causality_cfd::{
    BranchAccumulator, BranchOutcome, CaseRun, CompressibleMarchConfig, CompressiblePause, GateSeq,
    PhysicsError, Report, StudyView, TableRow, max_bond, quantize_2d,
};
use deep_causality_haft::LogSize;
use deep_causality_tensor::CausalTensor;

// ── The case axis: bank commands ──────────────────────────────────────────────────────────────

/// One commanded bank of the counterfactual study: the world name it flies under, the commanded
/// degrees, and — for the refinement round only — the shared aim point carried from the coarse
/// round so the two rounds score against the same target. The coarse round leaves `aim` unset and
/// derives it from the ballistic branch.
#[derive(Debug, Clone)]
pub struct BankCommand {
    pub name: &'static str,
    pub deg: FloatType,
    pub aim: Option<[FloatType; 3]>,
}

/// The coarse sweep's bank commands, keyed by commanded degrees (zero is the ballistic reference).
pub fn coarse_commands() -> Vec<BankCommand> {
    const NAMES: [&str; 6] = [
        "bank_00_deg",
        "bank_05_deg",
        "bank_10_deg",
        "bank_15_deg",
        "bank_20_deg",
        "bank_40_deg",
    ];
    BANK_ANGLES_DEG
        .iter()
        .zip(NAMES)
        .map(|(&deg, name)| BankCommand {
            name,
            deg,
            aim: None,
        })
        .collect()
}

/// The fine sweep's bank commands: [`FINE_SPAN_STEPS`] steps up and down from the coarse winner at
/// [`FINE_STEP_DEG`] resolution, clamped into the coarse span, carrying the coarse round's aim so
/// the rounds stay comparable. The coarse winner flies again in the middle, so the refinement can
/// only confirm or improve it.
///
/// # Errors
/// Never fails today; returns `Result` to fit the `refine` seam (a real refinement could reject a
/// degenerate round).
pub fn fine_candidates(coarse: &[BranchRow]) -> Result<Vec<BankCommand>, PhysicsError> {
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
    let winner = &coarse[pick_committed(coarse)];
    let aim = winner.aim;
    let lo = BANK_ANGLES_DEG[0];
    let hi = BANK_ANGLES_DEG[BANK_ANGLES_DEG.len() - 1];
    Ok(NAMES
        .iter()
        .enumerate()
        .map(|(k, &name)| {
            let offset = (k as FloatType - FINE_SPAN_STEPS as FloatType) * ft(FINE_STEP_DEG);
            let deg = (winner.bank_deg + offset).clamp(lo, hi);
            BankCommand {
                name,
                deg,
                aim: Some(aim),
            }
        })
        .collect())
}

/// A named descent world carrying its commanded bank as a world-published constant — the branch
/// world one bank command flies. The counterfactual branches differ *only* in that command.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn descent_world(
    name: &'static str,
    bank_deg: FloatType,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world::descent_world(
        name,
        world::standard_atmosphere(),
        STEPS,
        &[("commanded_bank", utils::rad(bank_deg))],
    )
}

/// Bind one bank command to its branch world — the config/execution seam of the fork study.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn bank_world(cmd: &BankCommand) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    descent_world(cmd.name, cmd.deg)
}

/// Rebuild the committed branch's world so the diagnostic legs can fly it. Deterministic from the
/// committed row's name and command.
///
/// # Errors
/// Propagates builder and codec failures.
pub fn committed_world(
    row: &BranchRow,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    descent_world(row.world_name, row.bank_deg)
}

// ── The reduction: one scored row per branch ──────────────────────────────────────────────────

/// One scored counterfactual branch — the row the reduction produces and the campaign gates read.
#[derive(Debug, Clone)]
pub struct BranchRow {
    pub bank_deg: FloatType,
    pub world_name: &'static str,
    pub outcome: BranchOutcome<FloatType>,
    /// The t²-law dead-reckoning proxy, printed as a cross-check beside the trajectory-derived
    /// miss: `½·|b|·dwell²` with `|b|` the accelerometer-bias magnitude.
    pub t2_miss_m: FloatType,
    /// The branch's terminal truth position (from the report's terminal trajectory states).
    pub terminal: [FloatType; 3],
    /// The shared aim point this branch scored against (carried into the fine round).
    pub aim: [FloatType; 3],
    pub has_alternation_marker: bool,
    /// The peak evolved electron density over the branch window (printed beside the outcome).
    pub ne_peak: FloatType,
    /// The branch's final evolved fields (T_tr and n_tot); the compression witness re-quantizes
    /// them.
    pub report_final: (Vec<FloatType>, Vec<FloatType>),
}

impl TableRow for BranchRow {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[
        ("bank", "deg"),
        ("peak_heat", "W/m2"),
        ("thermal_load", "J/m2"),
        ("dwell", "s"),
        ("miss_traj", "m"),
        ("miss_t2", "m"),
        ("peak_ne", "m^-3"),
    ];
    fn cells(&self) -> Vec<FloatType> {
        vec![
            self.bank_deg,
            self.outcome.peak_heat_flux,
            self.outcome.thermal_load,
            self.outcome.blackout_dwell,
            self.outcome.miss_distance,
            self.t2_miss_m,
            self.ne_peak,
        ]
    }
}

/// The aim point of the branch study: the ballistic (zero-bank) terminal state offset cross-range
/// in the direction a positive bank pushes toward, so steering is what closes it.
pub fn aim_point(ballistic_terminal: [FloatType; 3]) -> [FloatType; 3] {
    [
        ballistic_terminal[0],
        ballistic_terminal[1],
        ballistic_terminal[2] - ft(AIM_CROSS_RANGE_M),
    ]
}

/// The terminal truth position on a branch report.
pub fn terminal_position(report: &Report<FloatType>) -> [FloatType; 3] {
    let truth = report.series("final_truth_state").unwrap_or(&[]);
    core::array::from_fn(|i| truth.get(i).copied().unwrap_or_else(|| ft(f64::NAN)))
}

/// The collective reduction of a round: one row per branch, in case order. The shared aim point is
/// the carried one (the fine round) or the ballistic branch's terminal (the coarse round), so both
/// rounds score against the same target. Every branch's sensed heating and denial fold through the
/// [`BranchAccumulator`]; the close is the trajectory-derived miss to the aim.
///
/// # Errors
/// Fails only if a coarse round has no ballistic (zero-bank) branch to set the aim from.
pub fn score_branches(
    runs: &[CaseRun<'_, BankCommand, CompressibleMarchConfig<FloatType>, FloatType>],
) -> Result<Vec<BranchRow>, PhysicsError> {
    let aim = if let Some(a) = runs.iter().find_map(|r| r.case().aim) {
        a
    } else {
        let ballistic = runs.iter().find(|r| r.case().deg == 0.0).ok_or_else(|| {
            PhysicsError::CalculationError(
                "coarse round has no ballistic (0 deg) branch to set the aim point".into(),
            )
        })?;
        aim_point(terminal_position(ballistic.report()))
    };
    Ok(runs
        .iter()
        .map(|r| score_one(r.case(), r.report(), aim))
        .collect())
}

/// Score one branch report into a [`BranchRow`] against the shared aim point.
fn score_one(cmd: &BankCommand, report: &Report<FloatType>, aim: [FloatType; 3]) -> BranchRow {
    let heat = report.series("heat_flux").unwrap_or(&[]);
    let wp = report.series("plasma_frequency").unwrap_or(&[]);
    let band = ft(COMMS_BAND_RAD_S);
    let mut acc = BranchAccumulator::new(utils::rad(cmd.deg));
    for (i, &q) in heat.iter().enumerate() {
        let denied = wp.get(i).is_some_and(|&w| w > band);
        acc.observe(q, denied, ft(DT_FLIGHT));
    }
    let dwell = report
        .series("blackout_dwell")
        .and_then(|d| d.first().copied())
        .unwrap_or_else(|| ft(0.0));
    let bias: [FloatType; 3] = core::array::from_fn(|i| ft(IMU_ACCEL_BIAS[i]));
    let t2_miss_m = ft(0.5) * norm3(bias) * dwell * dwell;
    let terminal = terminal_position(report);
    let outcome = acc.finish_at(terminal, aim);
    BranchRow {
        bank_deg: cmd.deg,
        world_name: cmd.name,
        outcome,
        t2_miss_m,
        terminal,
        aim,
        has_alternation_marker: report
            .effect_log()
            .map(|l| format!("{l}").contains("!!ContextAlternation!!"))
            .unwrap_or(false),
        ne_peak: report
            .series("n_e")
            .map(utils::peak)
            .unwrap_or_else(|| ft(0.0)),
        report_final: (
            report.final_field().unwrap_or(&[]).to_vec(),
            report.series("final_n_tot").unwrap_or(&[]).to_vec(),
        ),
    }
}

/// The committed branch: minimum trajectory-derived miss distance. That is this corridor's scoring
/// rule; a flight design would weight all four outcome components.
pub fn pick_committed(rows: &[BranchRow]) -> usize {
    rows.iter()
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

/// The tensor-compression witness: re-quantize a branch's final evolved fields under the corridor's
/// round policy and read the peak bond dimension against the dense grid.
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

// ── The per-leg witness the trajectory gates read ─────────────────────────────────────────────

/// The per-leg witness the leg gates read, taken from a paused march's carried field.
#[derive(Debug, Clone)]
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

/// Snapshot a paused leg: flight state, regime, plasma, loads, and the navigation error against the
/// carried truth state.
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
        _ => ft(f64::NAN),
    };
    LegSnapshot {
        name: name.to_string(),
        steps: pause.step(),
        errored: pause.error().is_some(),
        altitude_km: utils::scalar0(field, "flight_altitude") / ft(1000.0),
        mach: utils::scalar0(field, "flight_mach"),
        regime_model: regime.map(|r| r.model.name()).unwrap_or("unclassified"),
        knudsen: regime.map(|r| r.knudsen).unwrap_or_else(|| ft(f64::NAN)),
        gnss_denied: regime.map(|r| r.gnss_denied).unwrap_or(false),
        ne_peak: field
            .scalar("n_e")
            .map(utils::peak)
            .unwrap_or_else(|| ft(0.0)),
        plasma_frequency: regime
            .map(|r| r.plasma_frequency)
            .unwrap_or_else(|| ft(0.0)),
        heat_flux: utils::scalar0(field, "heat_flux"),
        g_load: utils::scalar0(field, "g_load"),
        nav_err_m,
        nav_var: nav
            .map(|e| e.position_variance())
            .unwrap_or_else(|| ft(0.0)),
        log_entries: field.log().len(),
    }
}

/// Everything the trajectory (leg) gates read: the four flown legs plus the run-level witnesses.
#[derive(Debug, Clone)]
pub struct LegSet {
    pub leg1: LegSnapshot,
    pub leg2: LegSnapshot,
    pub leg3: LegSnapshot,
    pub leg4: LegSnapshot,
    pub rebuilds: usize,
    pub elapsed_s: FloatType,
    /// The rendered provenance log of the full descent (the regime-transition witness).
    pub regime_log: String,
}

// ── The campaign gating sequence: over the branch rows (fine round) and prior rounds ──────────

/// The campaign sequence: the counterfactual study's own validation — real steering, tensor
/// compression, guidance precision from the sweep, and the refinement improving the coarse winner.
/// Reads `view.rows()` (the fine round) and `view.rounds()[0]` (the coarse round).
pub fn corridor_gates() -> GateSeq<BranchRow> {
    GateSeq::new("bank-angle corridor")
        .gate("(4c) counterfactual steering", gate_steering)
        .gate("(4d) tensor compression", gate_compression)
        .gate("(4e) guidance precision from the sweep", gate_guidance)
        .gate("(4f) fine sweep refines the coarse winner", gate_refinement)
}

fn gate_steering(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let fine = v.rows();
    let coarse = v.rounds().first().map(Vec::as_slice).unwrap_or(&[]);
    let committed = pick_committed(coarse);
    let zero_bank = coarse.iter().position(|b| b.bank_deg == 0.0).unwrap_or(0);
    let sep: [FloatType; 3] =
        core::array::from_fn(|i| coarse[committed].terminal[i] - coarse[zero_bank].terminal[i]);
    let divergence = norm3(sep);
    let misses: Vec<FloatType> = coarse.iter().map(|b| b.outcome.miss_distance).collect();
    let hi = misses.iter().copied().fold(ft(f64::MIN), FloatType::max);
    let lo = misses.iter().copied().fold(ft(f64::MAX), FloatType::min);
    let pass = coarse.len() >= 2
        && coarse.iter().all(|b| b.has_alternation_marker)
        && fine.iter().all(|b| b.has_alternation_marker)
        && divergence > ft(DIVERGENCE_MIN_M)
        && hi > lo;
    (
        pass,
        format!(
            "{} alternated worlds ({} coarse + {} fine) from one shared onset; \
             coarse-winner-vs-ballistic terminal separation {:.2} m; coarse miss spread \
             [{:.2}, {:.2}] m",
            coarse.len() + fine.len(),
            coarse.len(),
            fine.len(),
            divergence,
            lo,
            hi,
        ),
    )
}

fn gate_compression(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let fine = v.rows();
    let committed = &fine[pick_committed(fine)];
    let (bond, dense) = compression_witness(&committed.report_final);
    (
        (1..=CAP).contains(&bond),
        format!(
            "the committed branch's final evolved state re-quantizes at peak bond {bond} (cap \
             {CAP}) vs the dense {dense}-cell grid",
        ),
    )
}

fn gate_guidance(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let fine = v.rows();
    let coarse = v.rounds().first().map(Vec::as_slice).unwrap_or(&[]);
    let zero_bank = coarse.iter().position(|b| b.bank_deg == 0.0).unwrap_or(0);
    let ballistic_miss = coarse[zero_bank].outcome.miss_distance;
    let committed = &fine[pick_committed(fine)];
    let committed_miss = committed.outcome.miss_distance;
    (
        committed_miss * ft(MISS_IMPROVEMENT_FACTOR) <= ballistic_miss,
        format!(
            "committed {:.1} deg lands {:.2} m off the aim vs the ballistic {:.2} m \
             ({:.1}x better; gate requires {MISS_IMPROVEMENT_FACTOR:.0}x)",
            committed.bank_deg,
            committed_miss,
            ballistic_miss,
            ballistic_miss / committed_miss,
        ),
    )
}

/// (4f) The fine sweep is a genuine refinement of the coarse winner: it brackets that winner on both
/// sides, and its branches flew distinct worlds.
///
/// The previous predicate was `min(fine miss) <= coarse-winner miss`. The fine candidate list always
/// contains the coarse winner's exact bank angle (offset 0 at `k = FINE_SPAN_STEPS`), flown from the
/// same paused onset with the same carried aim point, so the minimum is taken over a set that
/// includes the value it is compared against — the inequality held identically and the gate could
/// never fail. The improvement is still worth *printing*, so it stays in the detail line; what is
/// now *gated* are the two properties that are not structurally guaranteed.
///
/// BREAKING CONDITIONS: mis-centre `fine_candidates` (drop the ± span, or clamp the winner to an
/// envelope edge) and the bracketing check fails; collapse the fork so every branch returns the same
/// outcome and the distinct-worlds check fails.
fn gate_refinement(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let fine = v.rows();
    let coarse = v.rounds().first().map(Vec::as_slice).unwrap_or(&[]);
    let coarse_committed = &coarse[pick_committed(coarse)];
    let committed = &fine[pick_committed(fine)];

    // (a) The fine sweep brackets the coarse winner on both sides.
    let lo = fine
        .iter()
        .map(|b| b.bank_deg)
        .fold(ft(f64::MAX), FloatType::min);
    let hi = fine
        .iter()
        .map(|b| b.bank_deg)
        .fold(ft(f64::MIN), FloatType::max);
    let brackets = lo < coarse_committed.bank_deg && hi > coarse_committed.bank_deg;

    // (b) The branches flew distinct worlds: the fine misses are not all the same value.
    let m_lo = fine
        .iter()
        .map(|b| b.outcome.miss_distance)
        .fold(ft(f64::MAX), FloatType::min);
    let m_hi = fine
        .iter()
        .map(|b| b.outcome.miss_distance)
        .fold(ft(f64::MIN), FloatType::max);
    let distinct = m_hi > m_lo;

    (
        brackets && distinct,
        format!(
            "fine {:.1} deg at {:.2} m vs coarse {:.0} deg at {:.2} m; sweep brackets \
             [{:.1}, {:.1}] deg around the coarse winner, miss spread {:.3} m over {} branches",
            committed.bank_deg,
            committed.outcome.miss_distance,
            coarse_committed.bank_deg,
            coarse_committed.outcome.miss_distance,
            lo,
            hi,
            m_hi - m_lo,
            fine.len(),
        ),
    )
}

// ── The trajectory gating sequence: over the flown legs ───────────────────────────────────────

/// The trajectory sequence: the flown corridor's coupled validation — the blackout window as
/// events, the RAM-C II anchor, real INS drift and reacquisition, the regime crossing, the
/// multiphysics chain, bounded rebuilds, and the wall-clock budget. Reads the single [`LegSet`]
/// row through [`StudyView::of`].
pub fn leg_gates() -> GateSeq<LegSet> {
    GateSeq::new("corridor legs")
        .gate("(0) corridor integrity", gate_integrity)
        .gate("(1) flow-resolved blackout window", gate_window)
        .gate("(2) peak n_e vs the RAM-C II anchor", gate_anchor)
        .gate("(2b) blackout window altitudes", gate_window_altitudes)
        .gate("(3) real INS drift -> reacquisition", gate_drift)
        .gate("(4a) regime change", gate_regime_change)
        .gate("(4b) multiphysics chain", gate_multiphysics)
        .gate("(5a) bounded schedule rebuilds", gate_rebuilds)
        .gate("(5b) wall-clock budget", gate_wall_clock)
}

fn gate_integrity(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    let legs = [&l.leg1, &l.leg2, &l.leg3, &l.leg4];
    (
        legs.iter().all(|l| !l.errored),
        "no leg captured a step error (the envelope held)".into(),
    )
}

fn gate_window(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    let onset_found = l.leg1.gnss_denied && l.leg1.steps < STEPS;
    let exit_found = !l.leg3.gnss_denied && l.leg3.steps < STEPS;
    let dwell_steps = l.leg2.steps + l.leg3.steps;
    (
        onset_found && l.leg2.gnss_denied && exit_found && dwell_steps > 0,
        format!(
            "onset at step {} ({:.1} km), denied through the peak, exit at {:.1} km after a \
             {:.1} s dwell",
            l.leg1.steps,
            l.leg1.altitude_km,
            l.leg3.altitude_km,
            dwell_steps as FloatType * ft(DT_FLIGHT),
        ),
    )
}

fn gate_anchor(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    let ne_ok =
        (ft(RAMC_NE_REFERENCE / 5.0)..=ft(RAMC_NE_REFERENCE * 5.0)).contains(&l.leg2.ne_peak);
    (
        ne_ok,
        format!(
            "n_e = {:.3e} m^-3 at the {:.1} km passage, in [{:.1e}, {:.1e}] around the flight \
             anchor {:.0e} m^-3 (evolved state, uncalibrated finite-rate network; the \
             stagline-earned ±0.7-decade band)",
            l.leg2.ne_peak,
            l.leg2.altitude_km,
            RAMC_NE_REFERENCE / 5.0,
            RAMC_NE_REFERENCE * 5.0,
            RAMC_NE_REFERENCE,
        ),
    )
}

fn gate_window_altitudes(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    let (exit_lo, exit_hi) = EXIT_ALTITUDE_BAND_KM;
    let (ramc_lo, ramc_hi) = RAMC_EXIT_WINDOW_KM;
    (
        (ft(exit_lo)..=ft(exit_hi)).contains(&l.leg3.altitude_km),
        format!(
            "flow-resolved exit at {:.1} km (pinned band [{exit_lo:.0}, {exit_hi:.0}] km) vs the \
             RAM-C II {ramc_lo:.0}-{ramc_hi:.0} km flight window: the probe's light ballistic \
             bundle decelerates it below the ionization threshold higher, so the offset is \
             ballistics, not chemistry; onset predicted at {:.1} km",
            l.leg3.altitude_km, l.leg1.altitude_km,
        ),
    )
}

fn gate_drift(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    let drift = l.leg2.nav_err_m > l.leg1.nav_err_m && l.leg2.nav_var > l.leg1.nav_var;
    let reacq = l.leg4.nav_err_m < l.leg2.nav_err_m && l.leg4.nav_var < l.leg2.nav_var;
    (
        drift && reacq,
        format!(
            "err {:.4} m -> {:.4} m dead-reckoning to the peak passage, {:.4} m after \
             reacquisition; variance {:.3e} -> {:.3e} -> {:.3e} m^2",
            l.leg1.nav_err_m,
            l.leg2.nav_err_m,
            l.leg4.nav_err_m,
            l.leg1.nav_var,
            l.leg2.nav_var,
            l.leg4.nav_var,
        ),
    )
}

fn gate_regime_change(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    (
        l.regime_log.contains("regime -> slip") && l.regime_log.contains("regime -> continuum"),
        "the descent crossed a Knudsen band (slip -> continuum), logged as provenance events"
            .into(),
    )
}

fn gate_multiphysics(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    (
        l.leg2.ne_peak > ft(0.0)
            && l.leg2.heat_flux > ft(0.0)
            && l.leg2.g_load > ft(0.0)
            && l.leg2.nav_var > ft(0.0),
        "evolved flow -> reacting plasma -> steered aero force -> loads -> navigation all live in \
         one coupling"
            .into(),
    )
}

fn gate_rebuilds(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    (
        l.rebuilds <= MAX_REBUILDS,
        format!(
            "{} carrier rebuild(s) while following the descent (cap {MAX_REBUILDS})",
            l.rebuilds
        ),
    )
}

fn gate_wall_clock(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    (
        l.elapsed_s < ft(WALL_CLOCK_BUDGET_S),
        format!(
            "{:.1} s elapsed (budget {WALL_CLOCK_BUDGET_S:.0} s)",
            l.elapsed_s
        ),
    )
}
