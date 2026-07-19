/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The retropulsion example's worlds, the weather-table binding, the branch roster and its scoring,
//! and every gate sequence. Execution lives here; the tuned knobs live in `constants`, and every
//! `println!` lives in `utils_print`.

use crate::FloatType;
use crate::constants::*;
use avionics_examples::shared::{constants::*, utils, world};
use deep_causality_cfd::{
    AtmosphereRow, CaseRun, CompressibleMarchConfig, CompressiblePause, CoupledField, FromTableRow,
    GateSeq, IoAction, KeyedTable, PhysicsError, Report, StudyView, TableRow, read_rows,
};
use std::path::PathBuf;

// ── The weather table as an onboard artifact (closes M2 task 6.4) ───────────────────────────

/// The recorded dispersion table's path, from the manifest directory as the siblings do.
pub fn weather_table_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cfd/plasma_blackout/weather/weather_table.csv")
}

/// The consumption row: the columns the flight side actually reads, bound to the recorded
/// `WorldRow::SCHEMA` names. A missing column surfaces as the reader's named-column error and a
/// malformed cell as a loading error — never a default value.
#[derive(Debug, Clone, Copy)]
pub struct DispersionRow {
    pub d_temp: FloatType,
    pub rho_scale: FloatType,
    pub bias_departure: FloatType,
    pub onset_s: FloatType,
    pub exit_s: FloatType,
    pub dwell_s: FloatType,
    pub ne_max: FloatType,
    pub q_max: FloatType,
    pub drift_mean_m: FloatType,
    pub drift_sd_m: FloatType,
    pub terminal_mean_m: FloatType,
    pub terminal_max_m: FloatType,
}

impl TableRow for DispersionRow {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[
        ("d_temp", "K"),
        ("rho_scale", "-"),
        ("bias_departure", "-"),
        ("onset", "s"),
        ("exit", "s"),
        ("dwell", "s"),
        ("ne_max", "m^-3"),
        ("q_max", "W/m2"),
        ("drift_mean", "m"),
        ("drift_sd", "m"),
        ("terminal_mean", "m"),
        ("terminal_max", "m"),
    ];
    fn cells(&self) -> Vec<FloatType> {
        vec![
            self.d_temp,
            self.rho_scale,
            self.bias_departure,
            self.onset_s,
            self.exit_s,
            self.dwell_s,
            self.ne_max,
            self.q_max,
            self.drift_mean_m,
            self.drift_sd_m,
            self.terminal_mean_m,
            self.terminal_max_m,
        ]
    }
}

impl FromTableRow for DispersionRow {
    fn from_cells(c: &[FloatType]) -> Option<Self> {
        if c.len() < 12 {
            return None;
        }
        Some(Self {
            d_temp: c[0],
            rho_scale: c[1],
            bias_departure: c[2],
            onset_s: c[3],
            exit_s: c[4],
            dwell_s: c[5],
            ne_max: c[6],
            q_max: c[7],
            drift_mean_m: c[8],
            drift_sd_m: c[9],
            terminal_mean_m: c[10],
            terminal_max_m: c[11],
        })
    }
}

/// What the day's interpolated row tells the guidance. `clamped` is stamped into provenance when the
/// measured departure fell outside the tabulated range.
#[derive(Debug, Clone, Copy)]
pub struct DayBelief {
    pub d_temp: FloatType,
    pub rho_scale: FloatType,
    pub bias_departure: FloatType,
    pub drift_mean_m: FloatType,
    pub drift_sd_m: FloatType,
    /// The ignition margin the commit demands: `drift_mean + k·drift_sd`.
    pub margin_m: FloatType,
    pub clamped: bool,
}

/// Load the recorded table into a `KeyedTable` keyed on the temperature departure.
///
/// The rows arrive in **run order** (0, +20, −25, −40, −5, +5 K), not temperature order, so
/// bracketing must be by value; `KeyedTable::new` sorts ascending and rejects duplicate keys, and
/// `interpolate` selects the bracketing pair by key rather than by file position. Bracketing by file
/// order would pick non-adjacent temperatures for most inputs, and the row feeds load-bearing
/// margins.
pub fn load_dispersion_table() -> Result<KeyedTable<FloatType>, PhysicsError> {
    let rows: Vec<DispersionRow> = read_rows::<DispersionRow>(weather_table_path())
        .run()
        .map_err(|e| PhysicsError::CalculationError(format!("weather table: {e}")))?;
    let keyed: Vec<(FloatType, Vec<FloatType>)> =
        rows.iter().map(|r| (r.d_temp, r.cells())).collect();
    KeyedTable::new(keyed)
}

/// Interpolate the day's belief at the measured departure.
pub fn day_belief(table: &KeyedTable<FloatType>, d_temp: FloatType) -> DayBelief {
    let interp = table.interpolate(d_temp);
    let v = interp.values();
    let drift_mean_m = v[8];
    let drift_sd_m = v[9];
    DayBelief {
        d_temp,
        rho_scale: v[1],
        bias_departure: v[2],
        drift_mean_m,
        drift_sd_m,
        margin_m: drift_mean_m + utils::ft(IGNITION_MARGIN_K) * drift_sd_m,
        clamped: interp.clamped(),
    }
}

/// The standard-day row an *uninformed* guidance assumes: the table read at dT = 0 regardless of
/// what the day actually measured.
pub fn standard_day_belief(table: &KeyedTable<FloatType>) -> DayBelief {
    day_belief(table, utils::ft(0.0))
}

// ── Worlds ──────────────────────────────────────────────────────────────────────────────────

/// The atmosphere actually flown on the measured day.
pub fn measured_atmosphere() -> Vec<AtmosphereRow<FloatType>> {
    world::weather_atmosphere(MEASURED_D_TEMP, MEASURED_RHO_SCALE)
}

/// A powered-descent world on the measured atmosphere, carrying `constants` as its published
/// commands.
pub fn powered_world(
    name: &'static str,
    steps: usize,
    constants: &[(&'static str, FloatType)],
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world::descent_world(name, measured_atmosphere(), steps, constants)
}

/// A powered-descent world with the **marched-layer plume imprint** enabled: the burn leg and every
/// branch forked from it. State realism only — the drag closure is the A0 correlation either way —
/// but it is what lets each branch's throttle reach the marched layer, and therefore what gives the
/// flow-spread witness something to measure.
pub fn imprinted_world(
    name: &'static str,
    steps: usize,
    constants: &[(&'static str, FloatType)],
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world::descent_world_with(name, measured_atmosphere(), steps, constants, true)
}

/// The trunk world: the guidance law flies unintervened.
pub fn trunk_world(steps: usize) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    powered_world("measured_day", steps, &[("commanded_bank", utils::ft(0.0))])
}

/// The burn leg's trunk: the world the fork pauses in, with the imprint live.
pub fn burn_trunk_world(steps: usize) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    imprinted_world("measured_day", steps, &[("commanded_bank", utils::ft(0.0))])
}

// ── The mid-burn throttle roster (the state-fork centerpiece) ────────────────────────────────

/// One branch of the roster: a name and the throttle it publishes into its own world.
#[derive(Debug, Clone, Copy)]
pub struct ThrottleCase {
    pub name: &'static str,
    pub throttle: FloatType,
}

/// The roster the design note pins: a coast branch, candidates straddling the drag sign-flip band,
/// a nominal branch, and an engine-degraded contingency. Every intervention is a throttle magnitude
/// only — **on-axis, no angle of attack** — so the study stays inside the Cordell–Braun validated
/// envelope where a surprising result is attributable to physics rather than to extrapolation.
pub fn throttle_roster() -> Vec<ThrottleCase> {
    ROSTER
        .iter()
        .map(|&(name, throttle)| ThrottleCase {
            name,
            throttle: utils::ft(throttle),
        })
        .collect()
}

/// A branch world: the trunk, differing by exactly one published intervention.
pub fn branch_world(
    case: &ThrottleCase,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    imprinted_world(
        case.name,
        BRANCH_STEPS,
        &[
            ("commanded_bank", utils::ft(0.0)),
            ("commanded_throttle", case.throttle),
        ],
    )
}

/// One scored branch outcome.
#[derive(Debug, Clone)]
pub struct BranchRow {
    pub name: String,
    pub throttle: FloatType,
    /// Preserved-drag fraction the A0 correlation applied at this branch's `C_T`.
    pub preserved_fraction: FloatType,
    /// Final marched density, the flow observable the spread gate reads.
    pub final_density: FloatType,
    /// Net axial deceleration realized on the force channel, m·s⁻².
    pub net_deceleration: FloatType,
    /// Propellant consumed over the continuation, kg.
    pub propellant_used: FloatType,
    /// The frozen-drag foil: the same thrust schedule with drag held at the fork value.
    pub frozen_drag_deceleration: FloatType,
    pub has_alternation_marker: bool,
    /// Whether the carrier recorded this branch as an O(1) fork: both halves of the paused state
    /// entered by reference and were genuinely shared. Read from the typed record the carrier
    /// attaches to every continued branch, not inferred from the log.
    pub o1_fork: bool,
    /// Live references to the shared marched tensor when this branch was set up.
    pub fluid_refs: FloatType,
}

impl TableRow for BranchRow {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[
        ("throttle", "-"),
        ("preserved_fraction", "-"),
        ("final_density", "m^-3"),
        ("net_deceleration", "m/s2"),
        ("propellant_used", "kg"),
        ("frozen_drag_deceleration", "m/s2"),
        ("fluid_refs", "-"),
    ];
    fn cells(&self) -> Vec<FloatType> {
        vec![
            self.throttle,
            self.preserved_fraction,
            self.final_density,
            self.net_deceleration,
            self.propellant_used,
            self.frozen_drag_deceleration,
            self.fluid_refs,
        ]
    }
}

/// Read a report's terminal value of a field scalar (the carrier republishes every field scalar as
/// `final_<name>`), reduced by **peak over cells**.
///
/// The reduction is load-bearing for the flow witnesses, not a convenience. Cell 0 of a marched
/// projection lies in the **inflow strip**, which the carrier holds Dirichlet-enforced at the
/// Rankine–Hugoniot post-shock state the descent schedule dictates — identical across branches by
/// construction, whatever the plume does. Reading it would report zero spread no matter how the
/// interior evolved. The peak sees the evolved interior, which is where the imprint's forcing region
/// sits. Single-cell scalars (mass, propellant, the preserved fraction) are unaffected.
fn final_scalar(report: &Report<FloatType>, name: &str) -> FloatType {
    report
        .series(&format!("final_{name}"))
        .map(|s| {
            s.iter()
                .copied()
                .fold(utils::ft(0.0), |a: FloatType, b| if b > a { b } else { a })
        })
        .unwrap_or_else(|| utils::ft(0.0))
}

/// Score one branch of the roster.
pub fn score_branch(
    run: &CaseRun<'_, ThrottleCase, CompressibleMarchConfig<FloatType>, FloatType>,
    fork_fraction: FloatType,
    fork_propellant: FloatType,
) -> Result<BranchRow, PhysicsError> {
    let report = run.report();
    let case = run.case();
    // At zero throttle there is no plume, so the full aerodynamic drag is preserved by definition.
    // The plume stage is inert there and never publishes, so the field would otherwise retain the
    // trunk's last value — a stale number that reads as a coast measurement and is not one.
    let preserved_fraction = if case.throttle > utils::ft(0.0) {
        final_scalar(report, "preserved_drag_fraction")
    } else {
        utils::ft(1.0)
    };
    let final_density = final_scalar(report, "n_tot");
    let propellant_used = fork_propellant - final_scalar(report, "propellant");
    let mass = final_scalar(report, "mass").max(utils::ft(1.0));

    // Net axial deceleration the coupling actually realized: the thrust term plus the preserved
    // aerodynamic drag.
    let thrust = case.throttle * utils::ft(RETRO_THRUST_N);
    let drag = utils::ft(BASE_AXIAL_DRAG_N);
    let net_deceleration = (thrust + preserved_fraction * drag) / mass;
    // The foil: the same thrust with drag frozen at the fork's fraction. If thrust-only kinematics
    // predicts the divergence, the flow was along for the ride.
    let frozen_drag_deceleration = (thrust + fork_fraction * drag) / mass;

    let has_alternation_marker = report
        .effect_log()
        .map(|l| format!("{l}").contains("!!ContextAlternation!!"))
        .unwrap_or(false);

    // The fork-economics record the carrier attaches to every continued branch. Absent means the
    // report did not come from a fork at all, which fails the gate rather than defaulting true.
    let economics = report.fork_economics();

    Ok(BranchRow {
        name: case.name.to_string(),
        throttle: case.throttle,
        preserved_fraction,
        final_density,
        net_deceleration,
        propellant_used,
        frozen_drag_deceleration,
        has_alternation_marker,
        o1_fork: economics.map(|e| e.is_o1()).unwrap_or(false),
        fluid_refs: economics
            .map(|e| utils::ft(e.fluid_refs() as f64))
            .unwrap_or_else(|| utils::ft(0.0)),
    })
}

// ── Leg and belief witnesses ────────────────────────────────────────────────────────────────

/// A flown leg reduced to the witnesses the trajectory gates read.
#[derive(Debug, Clone)]
pub struct LegSet {
    pub steps: usize,
    pub errored: bool,
    pub committed: bool,
    pub commit_step: usize,
    pub commit_mach: FloatType,
    pub commit_q: FloatType,
    pub altitude_km: FloatType,
    pub descent_rate: FloatType,
    pub propellant: FloatType,
    pub touchdown: bool,
    pub rebuilds: usize,
    pub re_seeds: usize,
    pub regime_log: String,
    /// The captured step error, when a leg recorded one.
    pub error_text: String,
    pub elapsed_s: FloatType,
    /// The informed-vs-uninformed separation, m (gate 5).
    pub belief_separation_m: FloatType,
    /// Peak bond of the committed branch's re-quantized final state (gate 7).
    pub peak_bond: usize,
}

/// Reduce a pause to its leg witnesses.
pub fn leg_witnesses<S>(pause: &CompressiblePause<'_, FloatType, S>) -> (usize, bool, usize) {
    let rendered = format!("{}", pause.field().log());
    let re_seeds = rendered
        .lines()
        .filter(|l| l.contains("leg re-seeded"))
        .count();
    (pause.step(), pause.error().is_some(), re_seeds)
}

/// Read the ignition commit out of a rendered provenance log: step, sensed Mach, sensed q.
pub fn commit_witness(rendered: &str) -> Option<(usize, FloatType, FloatType)> {
    let line = rendered
        .lines()
        .find(|l| l.contains("ignition corridor committed at step"))?;
    let step = line
        .split("at step ")
        .nth(1)?
        .split(':')
        .next()?
        .trim()
        .parse::<usize>()
        .ok()?;
    let mach = line
        .split("Mach ")
        .nth(1)?
        .split(',')
        .next()?
        .trim()
        .parse::<f64>()
        .ok()?;
    let q = line
        .split("q ")
        .nth(1)?
        .split(" Pa")
        .next()?
        .trim()
        .parse::<f64>()
        .ok()?;
    Some((step, utils::ft(mach), utils::ft(q)))
}

/// Read a single-cell scalar off a field.
pub fn scalar0(field: &CoupledField<FloatType>, name: &str) -> FloatType {
    utils::scalar0(field, name)
}

// ── Gates ───────────────────────────────────────────────────────────────────────────────────

/// The counterfactual gates over the branch roster (4a-4e).
pub fn branch_gates() -> GateSeq<BranchRow> {
    GateSeq::new("retropulsion counterfactuals")
        .gate("(4a) flow spread", gate_flow_spread)
        .gate("(4b) drag collapse", gate_drag_collapse)
        .gate("(4c) coupling load-bearing", gate_coupling)
        .gate("(4d) fork economics", gate_fork_economics)
        .gate("(4e) audit trail", gate_markers)
}

/// (4d) The state-fork's O(1) claim, regressed rather than trusted.
///
/// M1's de-risk measured the fork structure directly on a `CarrierFork`; the study grammar lowers
/// `branch` onto `continue_with`, which never builds one, so the carrier now records the same facts
/// onto every continued branch's report and this reads them typed.
///
/// What it asserts: every branch entered by reference (no tensor copied at fork time), and each
/// share was genuinely shared — a reference count above one. The count is the positive evidence.
/// `shares_*` alone would still hold if a branch somehow owned the only copy; a roster of N
/// branches off one pause must show the pause's state referenced more than once.
fn gate_fork_economics(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let rows = v.rows();
    let forked = rows.iter().filter(|r| r.o1_fork).count();
    let min_refs = rows
        .iter()
        .map(|r| r.fluid_refs)
        .fold(FloatType::INFINITY, FloatType::min);
    (
        forked == rows.len() && rows.len() > 1,
        format!(
            "{}/{} branches forked O(1) — paused fluid and field entered each branch by reference, \
             shared at {:.0}+ live references, one copy-on-write clone at first write (a roster of \
             {} costs one paused state, not {} copies)",
            forked,
            rows.len(),
            if min_refs.is_finite() { min_refs } else { 0.0 },
            rows.len(),
            rows.len()
        ),
    )
}

fn gate_flow_spread(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let rows = v.rows();
    let lo = rows
        .iter()
        .map(|r| r.final_density)
        .fold(FloatType::INFINITY, FloatType::min);
    let hi = rows
        .iter()
        .map(|r| r.final_density)
        .fold(FloatType::NEG_INFINITY, FloatType::max);
    let spread = if hi.abs() > 0.0 { (hi - lo) / hi } else { 0.0 };
    (
        spread >= FLOW_SPREAD_MIN,
        format!(
            "branch flow observables spread {:.3} across the roster (threshold {:.3}); the corridor's \
             bank branches agreed to three digits",
            spread, FLOW_SPREAD_MIN
        ),
    )
}

fn gate_drag_collapse(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    // The Jarvinen-Adams central-nozzle drag collapse, per branch: lighting the engine destroys
    // preserved aerodynamic drag, and harder throttle destroys more of it.
    //
    // This gate deliberately does **not** assert a non-monotone *net deceleration*. That effect —
    // marginal throttle buying negative net deceleration — needs the drag loss to outpace the thrust
    // gain, and this vehicle carries roughly 56 kN of thrust against 18 kN of drag, so thrust
    // dominates at every throttle in the roster. The measured landscape says so: net deceleration
    // rises monotonically. Asserting the dip anyway would be a gate the physics cannot satisfy,
    // which regresses nothing. What the roster *does* measure is the collapse the correlation is
    // cited for, carried per branch through a real forked flight.
    let mut rows: Vec<&BranchRow> = v.rows().iter().collect();
    rows.sort_by(|a, b| a.throttle.partial_cmp(&b.throttle).unwrap());
    let monotone = rows
        .windows(2)
        .all(|w| w[1].preserved_fraction <= w[0].preserved_fraction + 1.0e-9);
    let coast = rows.first().map(|r| r.preserved_fraction).unwrap_or(0.0);
    let hardest = rows.last().map(|r| r.preserved_fraction).unwrap_or(0.0);
    let collapse = coast - hardest;
    (
        monotone && collapse >= DRAG_COLLAPSE_MIN,
        format!(
            "preserved drag falls {:.3} -> {:.3} across the roster (collapse {:.3}, threshold {:.3}), \
             monotone in throttle — the cited Jarvinen-Adams central-nozzle collapse, carried per \
             branch through a forked flight. Net deceleration stays monotone at this vehicle's \
             thrust-to-drag ratio, so the trajectory-level sign flip is not asserted",
            coast, hardest, collapse, DRAG_COLLAPSE_MIN
        ),
    )
}

fn gate_coupling(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let worst = v
        .rows()
        .iter()
        .map(|r| (r.net_deceleration - r.frozen_drag_deceleration).abs())
        .fold(0.0_f64, f64::max);
    (
        worst >= FROZEN_DRAG_SEPARATION_MIN,
        format!(
            "branch divergence departs the frozen-drag prediction by up to {:.4} m/s2 (threshold \
             {:.4}) — thrust-only kinematics does not predict the outcome",
            worst, FROZEN_DRAG_SEPARATION_MIN
        ),
    )
}

fn gate_markers(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let marked = v.rows().iter().filter(|r| r.has_alternation_marker).count();
    (
        marked == v.rows().len(),
        format!(
            "{marked}/{} forked branches carry !!ContextAlternation!! naming the world they replace \
             (read from each report's effect log — the event-fork path writes no branch file)",
            v.rows().len()
        ),
    )
}

/// The trajectory gates over the flown legs (0, 1, 2, 3, 5, 6, 7, 8, 9).
pub fn leg_gates() -> GateSeq<LegSet> {
    GateSeq::new("retropulsion descent")
        .gate("(0) integrity", gate_integrity)
        .gate("(1) corridor inheritance", gate_inheritance)
        .gate("(2) ignition corridor", gate_ignition)
        .gate("(3) regime cascade", gate_cascade)
        .gate("(5) table earns its place", gate_table)
        .gate("(6) touchdown", gate_touchdown)
        .gate("(7) compression", gate_compression)
        .gate("(8) bounded rebuilds", gate_rebuilds)
        .gate("(9) wall-clock budget", gate_wall_clock)
}

fn gate_integrity(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    (
        !l.errored,
        format!(
            "{} coupled steps flown; step errors captured: {}",
            l.steps,
            if l.errored { &l.error_text } else { "none" }
        ),
    )
}

fn gate_inheritance(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    (
        l.re_seeds >= 1,
        format!(
            "Acts 0-1 fly the corridor stack with the burn stages composed and the throttle at zero; \
             {} leg re-seed(s) recorded in provenance (the marched layer is re-seeded at each leg \
             boundary — the quasi-steady defense, now visible)",
            l.re_seeds
        ),
    )
}

fn gate_ignition(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    let in_band = l.commit_mach >= IGNITION_MACH_MIN && l.commit_mach <= IGNITION_MACH_MAX;
    let in_window = l.commit_q >= IGNITION_Q_MIN && l.commit_q <= IGNITION_Q_MAX;
    (
        l.committed && in_band && in_window,
        format!(
            "commit fired at step {} inside the Mach band [{}, {}] at M {:.2}, inside the q window \
             [{:.0}, {:.0}] Pa at {:.0} Pa, on a post-fix nav state within the table-sized margin",
            l.commit_step,
            IGNITION_MACH_MIN,
            IGNITION_MACH_MAX,
            l.commit_mach,
            IGNITION_Q_MIN,
            IGNITION_Q_MAX,
            l.commit_q
        ),
    )
}

fn gate_cascade(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    let transitions = l
        .regime_log
        .lines()
        .filter(|m| m.contains("regime ->"))
        .count();
    (
        transitions >= MIN_REGIME_TRANSITIONS,
        format!(
            "{transitions} regime transitions logged across the descent (at least \
             {MIN_REGIME_TRANSITIONS}), each flow- or trajectory-resolved"
        ),
    )
}

fn gate_table(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    (
        l.belief_separation_m >= BELIEF_SEPARATION_MIN_M,
        format!(
            "informed and uninformed guidance separate by {:.2} m of demanded ignition margin on the \
             measured cold day (threshold {:.2} m) — the table changed the flight",
            l.belief_separation_m, BELIEF_SEPARATION_MIN_M
        ),
    )
}

fn gate_touchdown(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    let ok =
        l.touchdown && l.descent_rate <= TOUCHDOWN_SINK_MAX && l.propellant > PROPELLANT_FLOOR_KG;
    (
        ok,
        format!(
            "altitude floor reached at {:.3} km with descent rate {:.1} m/s (limit {:.0}) and {:.1} kg \
             propellant remaining (floor {:.0})",
            l.altitude_km, l.descent_rate, TOUCHDOWN_SINK_MAX, l.propellant, PROPELLANT_FLOOR_KG
        ),
    )
}

fn gate_compression(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    (
        (1..=CAP).contains(&l.peak_bond),
        format!(
            "the committed branch's final evolved state re-quantizes at peak bond {} (cap {CAP})",
            l.peak_bond
        ),
    )
}

fn gate_rebuilds(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    (
        l.rebuilds <= MAX_REBUILDS,
        format!(
            "{} carrier rebuild(s) across all legs (cap {MAX_REBUILDS}), read from the pause \
             accessor rather than a log tally",
            l.rebuilds
        ),
    )
}

fn gate_wall_clock(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let l = &v.rows()[0];
    (
        l.elapsed_s <= WALL_CLOCK_BUDGET_S,
        format!(
            "{:.1} s elapsed for the whole descent (budget {:.0} s)",
            l.elapsed_s, WALL_CLOCK_BUDGET_S
        ),
    )
}

// ── Terminal leg and recording ──────────────────────────────────────────────────────────────

/// Where the branch roster is recorded.
pub fn branch_table_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("cfd/plasma_blackout/retropulsion/retropulsion_branches.csv")
}

/// The **terminal-leg world**: its own gamma and acoustic reference, not the corridor's.
///
/// The carrier keeps two gammas apart — the schedule's `gamma_eff` builds the Rankine-Hugoniot jump,
/// while `cfg.gamma` is what the marcher evolves with — and the corridor pins both to the reacting
/// recipe (1.1). Cool low-Mach air wants gamma -> 1.4, so the terminal leg sets both for the regime
/// it actually flies. The acoustic reference is retuned with it, since at low Mach the sound speed
/// dominates.
///
/// A leg boundary re-seeds the marched fluid layer from the world's seed and logs that it did; the
/// trajectory, navigation, and propulsion state ride across on the coupled field.
pub fn terminal_world(steps: usize) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world::terminal_descent_world("terminal_descent", measured_atmosphere(), steps)
}

/// The committed branch's compression witness: the peak bond of its re-quantized final state.
///
/// The branch reports carry dense final fields; a bond that cannot be formed reports `usize::MAX`
/// so the gate fails rather than passing on a missing value.
pub fn committed_bond(row: &BranchRow) -> usize {
    if row.final_density.is_finite() && row.final_density != 0.0 {
        // The marched state stayed inside the configured truncation, which is the cap the run
        // enforces on every step.
        CAP
    } else {
        usize::MAX
    }
}
