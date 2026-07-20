/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The retropulsion example's worlds, the weather-table binding, the branch roster and its scoring,
//! and every gate sequence. Execution lives here; the tuned knobs live in `constants`, and every
//! `println!` lives in `utils_print`.

use crate::FloatType;
use crate::constants::*;
use avionics_examples::shared::stages::FROZEN_DRAG_FRACTION_FIELD;
use avionics_examples::shared::{constants::*, utils, world};
use deep_causality_cfd::{
    AtmosphereRow, CaseRun, CompressibleMarchConfig, CoupledField, FromTableRow, GateSeq, IoAction,
    KeyedTable, PhysicsError, Report, STOPPING_BURN_ALTITUDE_FIELD, StudyView, TableRow, read_rows,
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
    /// Blackout onset the table predicts for this day, s after the descent start.
    pub onset_s: FloatType,
    /// Blackout dwell the table predicts for this day, s.
    pub dwell_s: FloatType,
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
        onset_s: v[3],
        dwell_s: v[5],
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
///
/// `rho_scale` is read from the day's interpolated dispersion row rather than hand-set beside it.
/// Two sources of truth for one physical quantity is how a printed belief and a flown atmosphere
/// drift apart while a two-decimal format hides the difference.
pub fn measured_atmosphere(rho_scale: FloatType) -> Vec<AtmosphereRow<FloatType>> {
    world::weather_atmosphere(MEASURED_D_TEMP, rho_scale)
}

/// A powered-descent world on the measured atmosphere, carrying `constants` as its published
/// commands.
pub fn powered_world(
    name: &'static str,
    steps: usize,
    rho_scale: FloatType,
    constants: &[(&'static str, FloatType)],
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world::descent_world(name, measured_atmosphere(rho_scale), steps, constants)
}

/// A powered-descent world with the **marched-layer plume imprint** enabled: the burn leg and every
/// branch forked from it. State realism only — the drag closure is the A0 correlation either way —
/// but it is what lets each branch's throttle reach the marched layer, and therefore what gives the
/// flow-spread witness something to measure.
pub fn imprinted_world(
    name: &'static str,
    steps: usize,
    rho_scale: FloatType,
    constants: &[(&'static str, FloatType)],
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world::descent_world_with(name, measured_atmosphere(rho_scale), steps, constants, true)
}

/// The trunk world: the guidance law flies unintervened.
pub fn trunk_world(
    steps: usize,
    rho_scale: FloatType,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    powered_world(
        "measured_day",
        steps,
        rho_scale,
        &[("commanded_bank", utils::ft(0.0))],
    )
}

/// The burn leg's trunk: the world the fork pauses in, with the imprint live.
pub fn burn_trunk_world(
    steps: usize,
    rho_scale: FloatType,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    imprinted_world(
        "measured_day",
        steps,
        rho_scale,
        &[("commanded_bank", utils::ft(0.0))],
    )
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
///
/// `fork_fraction` is the preserved-drag fraction the trunk carried at the fork. It is published
/// rather than baked into the coupling because the coupling stack is built for the burn leg, which
/// runs *before* the fork exists — so the fork's drag state can only reach the witness stage as a
/// world constant.
pub fn branch_world(
    case: &ThrottleCase,
    fork_fraction: FloatType,
    rho_scale: FloatType,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    imprinted_world(
        case.name,
        BRANCH_STEPS,
        rho_scale,
        &[
            ("commanded_bank", utils::ft(0.0)),
            ("commanded_throttle", case.throttle),
            (FROZEN_DRAG_FRACTION_FIELD, fork_fraction),
        ],
    )
}

/// One scored branch outcome.
///
/// Every field is read out of the branch's own report. The commanded throttle is carried alongside
/// the realized one because the safety envelope clamps commands, so the two differ whenever an axis
/// binds — and a table showing only the command presents one flight as several.
#[derive(Debug, Clone)]
pub struct BranchRow {
    pub name: String,
    /// The throttle this branch's world published — an input, not an outcome.
    pub commanded_throttle: FloatType,
    /// The throttle the propulsion stages actually flew, after the envelope clamped the command.
    pub realized_throttle: FloatType,
    /// Preserved-drag fraction the A0 correlation applied at this branch's `C_T`, or `None` when the
    /// closure applied no decrement at all — a coasting branch has no plume, and an absent fraction
    /// is the honest record of that. Substituting a literal `1.0` here would put a number the run
    /// never produced into the middle of two gates.
    pub preserved_fraction: Option<FloatType>,
    /// Final marched density, the flow observable the spread gate reads.
    pub final_density: FloatType,
    /// Axial deceleration read off the summed force channel at the end of the continuation, m·s⁻².
    pub net_deceleration: FloatType,
    /// Propellant consumed over the continuation, kg.
    pub propellant_used: FloatType,
    /// Velocity the branch actually shed along its flight path over the continuation, m·s⁻¹.
    pub dv_actual: FloatType,
    /// The frozen-drag foil over the same continuation: the branch's own thrust schedule with the
    /// preserved-drag fraction held at its value at the fork, m·s⁻¹.
    pub dv_frozen: FloatType,
    pub has_alternation_marker: bool,
    /// Whether the carrier recorded this branch as an O(1) fork: both halves of the paused state
    /// entered by reference. A guard against a source change rather than a run-time measurement.
    pub o1_fork: bool,
    /// How far this branch's rank grew past the rank the paused state carried at the fork.
    pub bond_growth: FloatType,
    /// The rank this branch's final marched state actually reached, as the carrier measured it.
    pub peak_bond: Option<usize>,
}

impl TableRow for BranchRow {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[
        ("commanded_throttle", "-"),
        ("realized_throttle", "-"),
        // NaN is the honest record for a branch whose closure applied no decrement: a
        // coasting branch has no plume, and any numeric sentinel here would be a magic value in the
        // middle of the gate that reads this column.
        ("preserved_fraction", "- (NaN: no decrement applied)"),
        ("final_density", "m^-3"),
        ("net_deceleration", "m/s2"),
        ("propellant_used", "kg"),
        ("dv_actual", "m/s"),
        ("dv_frozen", "m/s"),
        ("bond_growth", "-"),
    ];
    fn cells(&self) -> Vec<FloatType> {
        vec![
            self.commanded_throttle,
            self.realized_throttle,
            self.preserved_fraction.unwrap_or(FloatType::NAN),
            self.final_density,
            self.net_deceleration,
            self.propellant_used,
            self.dv_actual,
            self.dv_frozen,
            self.bond_growth,
        ]
    }
}

/// The missing-series error a witness read returns.
///
/// A default would be worse than an error here, and measurably so: a missing preserved-drag fraction
/// read as zero makes the drag-collapse gate *pass* with a full collapse, and a missing mass floored
/// to one kilogram inflates every deceleration and makes the coupling gate pass harder. Both failures
/// are silent, and both produce a plausible-looking row.
fn missing(name: &str) -> PhysicsError {
    PhysicsError::CalculationError(format!(
        "branch report carries no \"final_{name}\" series; scoring cannot substitute a value for it"
    ))
}

/// A finite check at the point of read, so no comparison downstream can see a not-a-number.
fn finite(name: &str, v: FloatType) -> Result<FloatType, PhysicsError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(PhysicsError::CalculationError(format!(
            "branch witness \"final_{name}\" is not finite ({v})"
        )))
    }
}

/// Read a **single-cell** field scalar off a report, preserving its sign.
///
/// A peak folded from zero cannot represent a negative value, and the preserved-drag fraction has a
/// negative branch: the SRP correlation reports a wake-type forebody force past a thrust coefficient
/// near two, which is exactly the sign-flip physics the counterfactual exists to find. Folding it to
/// zero would erase the effect and report the erasure as a measurement.
fn final_cell0(report: &Report<FloatType>, name: &str) -> Result<FloatType, PhysicsError> {
    let v = report
        .series(&format!("final_{name}"))
        .and_then(|s| s.first().copied())
        .ok_or_else(|| missing(name))?;
    finite(name, v)
}

/// Read a **per-cell marched** quantity off a report, reduced by peak over cells.
///
/// The reduction is load-bearing for the flow witnesses, not a convenience. Cell 0 of a marched
/// projection lies in the **inflow strip**, which the carrier holds Dirichlet-enforced at the
/// Rankine–Hugoniot post-shock state the descent schedule dictates — identical across branches by
/// construction, whatever the plume does. Reading it would report zero spread no matter how the
/// interior evolved. The peak sees the evolved interior, which is where the imprint's forcing region
/// sits.
fn final_peak(report: &Report<FloatType>, name: &str) -> Result<FloatType, PhysicsError> {
    let series = report
        .series(&format!("final_{name}"))
        .ok_or_else(|| missing(name))?;
    let v = series
        .iter()
        .copied()
        .reduce(|a, b| if b > a { b } else { a })
        .ok_or_else(|| missing(name))?;
    finite(name, v)
}

/// The trunk's state at the fork, so each branch is scored on what *it* changed rather than on the
/// descent it inherited.
#[derive(Debug, Clone, Copy)]
pub struct ForkState {
    /// Preserved-drag fraction the trunk carried at the fork — the value the foil freezes at.
    pub fraction: FloatType,
    pub propellant: FloatType,
    pub dv_actual: FloatType,
    pub dv_frozen: FloatType,
}

/// Score one branch of the roster, entirely from what the branch flew.
///
/// Nothing here re-derives an outcome from the roster's commanded throttle. The safety envelope
/// clamps commands against a dynamic thrust-coefficient ceiling, so several roster entries can fly
/// one identical trajectory; scoring the command would then report one flight as several, with a
/// spread that is the command's arithmetic rather than the flight's.
pub fn score_branch(
    run: &CaseRun<'_, ThrottleCase, CompressibleMarchConfig<FloatType>, FloatType>,
    fork: ForkState,
) -> Result<BranchRow, PhysicsError> {
    let report = run.report();
    let case = run.case();

    let realized_throttle = final_cell0(report, "realized_throttle")?;
    // Absent means the closure applied nothing this branch — no throttle, or outside its Mach band.
    // That is a state, not a missing measurement, so it is carried rather than defaulted or errored.
    let preserved_fraction = match report.series("final_preserved_drag_fraction") {
        Some(_) => Some(final_cell0(report, "preserved_drag_fraction")?),
        None => None,
    };
    let final_density = final_peak(report, "n_tot")?;
    let net_deceleration = final_cell0(report, "axial_accel")?;
    let propellant_used = fork.propellant - final_cell0(report, "propellant")?;
    let dv_actual = final_cell0(report, "dv_actual")? - fork.dv_actual;
    let dv_frozen = final_cell0(report, "dv_frozen")? - fork.dv_frozen;

    let has_alternation_marker = report.alternation_applied().unwrap_or(false);
    let economics = report.fork_economics();

    Ok(BranchRow {
        name: case.name.to_string(),
        commanded_throttle: case.throttle,
        realized_throttle,
        preserved_fraction,
        final_density,
        net_deceleration,
        propellant_used,
        dv_actual,
        dv_frozen,
        has_alternation_marker,
        // Absent economics means the report did not come from a fork at all, which fails the gate
        // rather than defaulting true.
        o1_fork: economics.map(|e| e.is_o1()).unwrap_or(false),
        bond_growth: report
            .bond_growth()
            .map(|g| utils::ft(g as f64))
            .unwrap_or_else(|| utils::ft(-1.0)),
        peak_bond: report.peak_bond(),
    })
}

// ── Leg and belief witnesses ────────────────────────────────────────────────────────────────

/// A flown leg reduced to the witnesses the trajectory gates read.
///
/// Every field is a typed read. Nothing here is recovered by rendering the provenance log and
/// splitting strings: a reworded message would make such a read report zero, and a gate would then
/// fail for a reason that has nothing to do with the flight.
#[derive(Debug, Clone)]
pub struct LegSet {
    pub steps: usize,
    /// Each leg's captured step error, named. Empty when every leg flew clean.
    pub leg_errors: Vec<(String, String)>,
    pub committed: bool,
    pub commit_step: FloatType,
    pub commit_mach: FloatType,
    pub commit_q: FloatType,
    /// Whether the navigation was aided on the committing step.
    pub commit_aided: bool,
    /// Navigated position uncertainty at the commit, m (one sigma).
    pub commit_sigma_m: FloatType,
    /// The margin the commit was required to satisfy, from the day's interpolated row.
    pub commit_margin_m: FloatType,
    pub altitude_km: FloatType,
    pub descent_rate: FloatType,
    pub propellant: FloatType,
    pub touchdown: bool,
    pub rebuilds: usize,
    pub re_seeds: FloatType,
    pub regime_transitions: FloatType,
    /// Acts 0-1 blackout witnesses, compared against what the dispersion table predicts for the
    /// measured day.
    pub onset_s: FloatType,
    pub dwell_s: FloatType,
    pub drift_denied_max_m: FloatType,
    /// The window the table predicted for this day, in the same units.
    pub predicted_onset_s: FloatType,
    pub predicted_dwell_s: FloatType,
    pub elapsed_s: FloatType,
    /// The fan-out's per-step wall-clock against the trunk's (gate 4g).
    pub step_cost_ratio: FloatType,
    /// What the two beliefs flew (gate 5).
    pub belief: BeliefOutcome,
    /// Whether the day's dispersion row clamped to the nearest tabulated key.
    pub belief_clamped: bool,
    /// Peak bond of the committed branch's re-quantized final state (gate 7), or `None` when the
    /// branch reported no rank.
    pub peak_bond: Option<usize>,
}

/// Read a single-cell scalar off a field.
pub fn scalar0(field: &CoupledField<FloatType>, name: &str) -> FloatType {
    utils::scalar0(field, name)
}

/// What the two beliefs actually flew, compared on outcomes rather than on the table arithmetic that
/// produced their margins.
///
/// The margin reaches the flight through the stopping burn's ignition altitude, so a guidance that
/// believes the day is more dispersed lights its landing burn higher and spends more propellant
/// arriving. Both numbers here are read off flown fields.
#[derive(Debug, Clone, Copy)]
pub struct BeliefOutcome {
    /// Altitude the informed world lit its stopping burn at, m.
    pub informed_ignition_m: FloatType,
    /// Altitude the uninformed world lit its stopping burn at, m.
    pub uninformed_ignition_m: FloatType,
    /// Separation between the two landing decisions, m.
    pub ignition_separation_m: FloatType,
    /// Propellant the informed world spent above what the uninformed one did, kg.
    ///
    /// This **is** the dispersion-sized reserve of design note §4's third job, measured rather than
    /// configured: the margin already buys it by starting the burn higher on a more-dispersed day.
    pub reserve_kg: FloatType,
    /// Descent rate each world arrived at, m/s.
    pub informed_contact_ms: FloatType,
    pub uninformed_contact_ms: FloatType,
}

/// Reduce the two flown terminal legs to their belief witnesses.
pub fn belief_outcome(
    informed: &CoupledField<FloatType>,
    uninformed: &CoupledField<FloatType>,
) -> BeliefOutcome {
    let informed_ignition_m = scalar0(informed, STOPPING_BURN_ALTITUDE_FIELD);
    let uninformed_ignition_m = scalar0(uninformed, STOPPING_BURN_ALTITUDE_FIELD);
    BeliefOutcome {
        informed_ignition_m,
        uninformed_ignition_m,
        ignition_separation_m: (informed_ignition_m - uninformed_ignition_m).abs(),
        reserve_kg: scalar0(uninformed, "propellant") - scalar0(informed, "propellant"),
        informed_contact_ms: scalar0(informed, "descent_rate"),
        uninformed_contact_ms: scalar0(uninformed, "descent_rate"),
    }
}

/// The fan-out's per-step wall-clock relative to the trunk's.
///
/// The fork's second economics question — a fork that is O(1) to take is only cheap if the branches
/// then march at the trunk's rate. The solver crate carries no clock and should not: timing belongs
/// to whoever owns the run, so the example times the leg it forked from and the fan-out it spawned.
///
/// Deliberately a **run-level** number rather than a per-branch column. Branches run concurrently
/// under the `parallel` feature, so a per-branch wall-clock would be an attribution of shared time
/// and would read as precision the measurement does not have. What the fan-out's own clock supports
/// is "one branch step cost this much more than one trunk step", which is the quantity design note
/// §10(4d) asks for.
pub fn step_cost_ratio(
    trunk_s: FloatType,
    trunk_steps: usize,
    fan_out_s: FloatType,
    branch_steps: usize,
) -> FloatType {
    if trunk_steps == 0 || branch_steps == 0 || trunk_s <= 0.0 {
        return utils::ft(0.0);
    }
    (fan_out_s / utils::ft(branch_steps as f64)) / (trunk_s / utils::ft(trunk_steps as f64))
}

// ── Gates ───────────────────────────────────────────────────────────────────────────────────

/// The counterfactual gates over the branch roster (4a-4f).
pub fn branch_gates() -> GateSeq<BranchRow> {
    GateSeq::new("retropulsion counterfactuals")
        .gate("(4a) flow spread", gate_flow_spread)
        .gate("(4b) drag collapse", gate_drag_collapse)
        .gate("(4c) coupling load-bearing", gate_coupling)
        .gate("(4d) fork economics", gate_fork_economics)
        .gate("(4e) audit trail", gate_markers)
        .gate("(4f) roster non-degeneracy", gate_roster_distinct)
}

/// (4f) The roster must fly distinct throttles.
///
/// The safety envelope clamps every command against a dynamic thrust-coefficient ceiling that moves
/// with the sensed dynamic pressure, so a roster written in commanded throttles can collapse onto one
/// realized throttle without anything saying so. Every downstream witness then agrees to full
/// precision across the collapsed branches while the recorded table still shows the distinct commands,
/// which reads as a spread that was never flown. This gate makes that collapse a run failure.
fn gate_roster_distinct(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let rows = v.rows();
    let mut collisions: Vec<String> = Vec::new();
    for (i, a) in rows.iter().enumerate() {
        for b in rows.iter().skip(i + 1) {
            if (a.realized_throttle - b.realized_throttle).abs() < ROSTER_THROTTLE_MIN_GAP {
                collisions.push(format!(
                    "{} and {} both flew {:.4}",
                    a.name, b.name, a.realized_throttle
                ));
            }
        }
    }
    (
        collisions.is_empty() && rows.len() > 1,
        if collisions.is_empty() {
            format!(
                "{} branches flew {} distinct throttles (minimum gap {:.3}) — the envelope admitted \
                 every commanded value, so each row describes its own flight",
                rows.len(),
                rows.len(),
                ROSTER_THROTTLE_MIN_GAP
            )
        } else {
            format!(
                "roster collapsed: {} — the envelope clamped distinct commands onto one flight, so \
                 the rows below it are one trajectory reported several times",
                collisions.join("; ")
            )
        },
    )
}

/// (4d) The state-fork's cost, regressed rather than trusted.
///
/// Two claims, of different kinds. `is_o1` is a **source-change guard**: both of its conjuncts
/// compare a clone against the `Arc` it was cloned from, so no input falsifies them, but an edit that
/// materializes the paused state instead of sharing it flips them and this fails. Bond growth is a
/// genuine **measurement**: it varies with the run, and it answers the half of the fork-economics
/// question that sharing cannot — whether a state that forks cheaply then stays cheap.
fn gate_fork_economics(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let rows = v.rows();
    let forked = rows.iter().filter(|r| r.o1_fork).count();
    let worst_growth = rows
        .iter()
        .map(|r| r.bond_growth)
        .fold(FloatType::NEG_INFINITY, FloatType::max);
    let measured = rows.iter().all(|r| r.bond_growth >= 0.0);
    (
        forked == rows.len() && rows.len() > 1 && measured && worst_growth <= MAX_BOND_GROWTH,
        format!(
            "{}/{} branches entered by reference (no tensor copied at fork time), and the worst \
             post-fork bond growth is {:.0} against a cap of {:.0}. The fan-out's step cost is \
             gate (4g), which is run-level because the branches are concurrent",
            forked,
            rows.len(),
            if worst_growth.is_finite() {
                worst_growth
            } else {
                -1.0
            },
            MAX_BOND_GROWTH
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
    let spread = if hi.is_finite() && hi.abs() > 0.0 {
        (hi - lo) / hi.abs()
    } else {
        0.0
    };
    (
        spread >= FLOW_SPREAD_MIN,
        format!(
            "branch flow observables spread {:.4} across the roster (threshold {:.4}); the corridor's \
             bank branches agreed to three digits",
            spread, FLOW_SPREAD_MIN
        ),
    )
}

fn gate_drag_collapse(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    // The Jarvinen-Adams central-nozzle drag collapse, carried per branch through a forked flight:
    // lighting the engine destroys preserved aerodynamic drag, and harder throttle destroys more.
    //
    // Ordered by **realized** throttle, since that is what set each branch's thrust coefficient. Only
    // branches whose closure actually applied a decrement take part: a coasting branch has no plume,
    // so it has no fraction to compare, and inventing one for it would decide the gate on a number
    // the run never produced.
    //
    // The fractions are read sign-preserving, so the correlation's negative branch survives to be
    // measured. That branch is the wake-type forebody force past a thrust coefficient near two, and
    // it is the sign-flip the design note asks this gate to find.
    let mut rows: Vec<(&BranchRow, FloatType)> = v
        .rows()
        .iter()
        .filter_map(|r| r.preserved_fraction.map(|f| (r, f)))
        .collect();
    rows.sort_by(|a, b| {
        a.0.realized_throttle
            .partial_cmp(&b.0.realized_throttle)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    if rows.len() < 2 {
        return (
            false,
            format!(
                "only {} branch(es) applied a drag decrement — a collapse needs at least two points",
                rows.len()
            ),
        );
    }
    let monotone = rows.windows(2).all(|w| w[1].1 <= w[0].1 + 1.0e-9);
    let softest = rows.first().map(|r| r.1).unwrap_or(0.0);
    let hardest = rows.last().map(|r| r.1).unwrap_or(0.0);
    let collapse = softest - hardest;
    let reversed = rows.iter().any(|r| r.1 < 0.0);
    let coasting = v.rows().len() - rows.len();

    // ── The sign flip §3.2 asks this gate to find. ──
    //
    // Ordered by realized throttle over the **whole** roster including the coast branch, since the
    // effect is precisely that lighting the engine can buy *less* net deceleration than coasting: in
    // the low-thrust-coefficient band the plume destroys preserved drag about as fast as thrust
    // replaces it. Measured from the trajectory-derived deceleration each branch actually realized,
    // not predicted from the correlation — the correlation is the cross-check.
    let mut by_throttle: Vec<&BranchRow> = v.rows().iter().collect();
    by_throttle.sort_by(|a, b| a.realized_throttle.total_cmp(&b.realized_throttle));
    let decel: Vec<FloatType> = by_throttle.iter().map(|r| r.net_deceleration).collect();
    let dip = decel
        .iter()
        .enumerate()
        .skip(1)
        .take(decel.len().saturating_sub(2))
        .filter(|(i, d)| **d < decel[i - 1] && **d < decel[i + 1])
        .map(|(i, _)| i)
        .next();
    let sign_flip = match dip {
        Some(i) => format!(
            "; net deceleration is non-monotone in throttle with its minimum at the {} branch \
             (throttle {:.2}, {:.3} m/s2 against {:.3} m/s2 coasting) — the drag sign flip",
            by_throttle[i].name, by_throttle[i].realized_throttle, decel[i], decel[0]
        ),
        None => "; net deceleration is monotone in throttle, so no sign flip is present in this \
                 roster's band"
            .to_string(),
    };

    (
        monotone && collapse >= DRAG_COLLAPSE_MIN && dip.is_some(),
        format!(
            "preserved drag falls {:.4} -> {:.4} across the {} burning branches' realized throttles \
             (collapse {:.4}, threshold {:.4}), monotone in throttle; {} coasting branch(es) applied \
             no decrement{}",
            softest,
            hardest,
            rows.len(),
            collapse,
            DRAG_COLLAPSE_MIN,
            coasting,
            if reversed {
                " — and the harder branches reach the correlation's negative, wake-type branch"
            } else {
                ""
            }
        ) + &sign_flip,
    )
}

/// (4c) The coupling is load-bearing, measured on trajectories.
///
/// Both quantities are velocity increments accumulated step by step over the same continuation: the
/// one the branch actually shed, and the one it would have shed under its own thrust schedule with
/// the drag closure frozen at the fork's fraction. They differ only through the closure, so their
/// separation is what the coupling contributed. Comparing two closed forms that share a thrust term
/// would instead cancel that term identically and restate the drag gate.
fn gate_coupling(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let worst = v
        .rows()
        .iter()
        .map(|r| (r.dv_actual - r.dv_frozen).abs())
        .fold(0.0_f64, f64::max);
    (
        worst >= FROZEN_DRAG_SEPARATION_MIN,
        format!(
            "branch trajectories depart the frozen-drag prediction by up to {:.4} m/s over the \
             continuation (threshold {:.4}) — thrust-only kinematics does not predict the outcome",
            worst, FROZEN_DRAG_SEPARATION_MIN
        ),
    )
}

/// (4e) Every branch flew a world alternation that was actually applied.
///
/// Read from the typed flag, not from a marker search: the carrier writes the same
/// `!!ContextAlternation!!` marker on the path where it *refuses* to apply an alternation, so a
/// substring match reports a refused branch as an alternated one.
fn gate_markers(v: &StudyView<'_, BranchRow>) -> (bool, String) {
    let marked = v.rows().iter().filter(|r| r.has_alternation_marker).count();
    (
        marked == v.rows().len() && !v.rows().is_empty(),
        format!(
            "{marked}/{} branches record an applied context alternation naming the world they \
             replace (the typed flag, which the carrier's refusal path sets false)",
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
        .gate("(4g) fork step cost", gate_step_cost)
        .gate("(5) table earns its place", gate_table)
        .gate("(6) touchdown", gate_touchdown)
        .gate("(7) compression", gate_compression)
        .gate("(8) bounded rebuilds", gate_rebuilds)
        .gate("(9) wall-clock budget", gate_wall_clock)
}

fn gate_integrity(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let Some(l) = v.rows().first() else {
        return (false, "no leg witnesses were recorded".into());
    };
    (
        l.leg_errors.is_empty(),
        if l.leg_errors.is_empty() {
            format!(
                "{} coupled steps flown across four legs; no leg captured a step error",
                l.steps
            )
        } else {
            format!(
                "step errors captured: {}",
                l.leg_errors
                    .iter()
                    .map(|(leg, e)| format!("{leg}: {e}"))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        },
    )
}

/// (1) Acts 0-1 fly the blackout window the day's dispersion row predicts.
///
/// The design note asks this gate for *bit-identical* reproduction of the corridor's window. That
/// requirement and this example's own §4 premise are incompatible, and the incompatibility is real
/// physics rather than a wiring mistake: the example flies the **measured cold day**, and a colder,
/// denser atmosphere ionizes earlier and dwells longer. Its window therefore cannot match the
/// corridor's standard-day window, and a gate demanding that it does would be asking the descent to
/// ignore the weather it was built to consume.
///
/// What inheritance means here is that Acts 0-1 fly the corridor *stack* — burn stages composed and
/// contractually inert at zero throttle — over the day actually flown. That is checked two ways, and
/// both can fail:
///
/// * the flown blackout window matches the `onset` and `dwell` the dispersion table records for this
///   temperature departure. The table was generated by the sibling weather example from the same
///   coupling stack, so agreement is a statement that the stack still behaves as it did — and it puts
///   the table's own columns under test rather than only its drift row;
/// * no propellant was consumed and no ignition latched before the corridor leg ended, which is the
///   zero-throttle inertness contract the burn stack rests on.
fn gate_inheritance(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let Some(l) = v.rows().first() else {
        return (false, "no leg witnesses were recorded".into());
    };
    let onset_err = (l.onset_s - l.predicted_onset_s).abs();
    let dwell_err = (l.dwell_s - l.predicted_dwell_s).abs();
    let ok = onset_err <= WINDOW_PREDICTION_TOL_S && dwell_err <= WINDOW_PREDICTION_TOL_S;
    (
        ok,
        format!(
            "Acts 0-1 fly the corridor stack with the burn stages composed and inert: blackout onset \
             at {:.2} s against the table's {:.2} s for this day (error {:.3} s) and dwell {:.2} s \
             against {:.2} s (error {:.3} s), tolerance {:.2} s; {:.0} leg re-seed(s), peak \
             dead-reckoning drift {:.2} m",
            l.onset_s,
            l.predicted_onset_s,
            onset_err,
            l.dwell_s,
            l.predicted_dwell_s,
            dwell_err,
            WINDOW_PREDICTION_TOL_S,
            l.re_seeds,
            l.drift_denied_max_m
        ),
    )
}

/// (2) The ignition commit satisfied the conditions its own predicate does **not** guarantee.
///
/// The Mach band and the dynamic-pressure window are the corridor predicate's own preconditions, so
/// a commit witness can only exist for values inside them — re-asserting those reduces the gate to
/// "a commit happened". What the predicate's existence does not tell a reader is whether the state it
/// committed on was aided and inside the day's margin, so that is what this checks.
fn gate_ignition(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let Some(l) = v.rows().first() else {
        return (false, "no leg witnesses were recorded".into());
    };
    let inside_margin = l.commit_sigma_m <= l.commit_margin_m;
    (
        l.committed && l.commit_aided && inside_margin,
        format!(
            "commit fired at step {:.0} on {} navigation at sigma {:.3} m against the day's \
             table-sized margin of {:.2} m (Mach {:.2}, q {:.0} Pa — the corridor predicate's own \
             preconditions, not re-asserted here)",
            l.commit_step,
            if l.commit_aided {
                "aided"
            } else {
                "dead-reckoning"
            },
            l.commit_sigma_m,
            l.commit_margin_m,
            l.commit_mach,
            l.commit_q
        ),
    )
}

fn gate_cascade(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let Some(l) = v.rows().first() else {
        return (false, "no leg witnesses were recorded".into());
    };
    (
        l.regime_transitions >= utils::ft(MIN_REGIME_TRANSITIONS as f64),
        format!(
            "{:.0} regime transitions logged across the descent (at least {MIN_REGIME_TRANSITIONS}), \
             counted by the classifier rather than tallied from a rendered log",
            l.regime_transitions
        ),
    )
}

/// (4g) The forked branches march at the trunk's rate.
///
/// Run-level rather than per-branch: the branches are concurrent, so their wall-clock is shared and a
/// per-branch column would report precision the measurement does not have. Completes design note
/// §10(4d), whose sharing and bond-growth halves gate (4d) carries.
fn gate_step_cost(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let Some(l) = v.rows().first() else {
        return (false, "no leg witnesses were recorded".into());
    };
    (
        l.step_cost_ratio > 0.0 && l.step_cost_ratio <= MAX_STEP_COST_RATIO,
        format!(
            "the branch fan-out marched at {:.2}x the trunk's per-step cost (cap {:.2}) — the roster \
             costs about one branch's time rather than the sum, which is what an O(1) fork onto \
             concurrent branches is for",
            l.step_cost_ratio, MAX_STEP_COST_RATIO
        ),
    )
}

/// (5) The table changed the flight.
///
/// Compares two **flown** worlds, not two lookups. The previous form subtracted two interpolations of
/// one CSV before any march and reported the difference as evidence that the table changed something;
/// that number is invariant to the entire descent, and the margin it described never bound, because
/// the navigated sigma was two orders of magnitude inside it.
///
/// Where the margin does bind is the stopping burn: `ignition_altitude_kernel` adds it to the
/// stopping distance, so a guidance that believes the day is more dispersed lights its landing burn
/// higher. The separation of those two landing decisions, and the propellant the informed world spent
/// to buy it, are the flown consequence.
fn gate_table(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let Some(l) = v.rows().first() else {
        return (false, "no leg witnesses were recorded".into());
    };
    let b = l.belief;
    let separated = b.ignition_separation_m >= BELIEF_SEPARATION_MIN_M;
    (
        separated && !l.belief_clamped,
        format!(
            "informed and uninformed guidance lit the landing burn {:.2} m apart ({:.2} m vs {:.2} m, \
             threshold {:.2} m) and arrived at {:.2} vs {:.2} m/s; the informed world spent {:.2} kg \
             more propellant, which is the dispersion-sized reserve measured rather than configured{}",
            b.ignition_separation_m,
            b.informed_ignition_m,
            b.uninformed_ignition_m,
            BELIEF_SEPARATION_MIN_M,
            b.informed_contact_ms,
            b.uninformed_contact_ms,
            b.reserve_kg,
            if l.belief_clamped {
                " — but the measured departure fell outside the tabulated range and clamped, so the \
                 row is an extrapolation rather than an interpolation"
            } else {
                ""
            }
        ),
    )
}

/// (6) The vehicle arrives at its commanded contact condition.
///
/// Bounded on **both** sides. A one-sided ceiling admits an undershoot and admits a hover, which are
/// the outcomes this gate's tracking claim says it detects — a guidance that nulls its velocity above
/// the deck passes an upper bound while failing to land the way it was commanded to.
fn gate_touchdown(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let Some(l) = v.rows().first() else {
        return (false, "no leg witnesses were recorded".into());
    };
    let lo = utils::ft(CONTACT_SPEED_MS) - utils::ft(TOUCHDOWN_SINK_TOL);
    let hi = utils::ft(CONTACT_SPEED_MS) + utils::ft(TOUCHDOWN_SINK_TOL);
    let tracked = l.descent_rate >= lo && l.descent_rate <= hi;
    (
        l.touchdown && tracked && l.propellant > PROPELLANT_FLOOR_KG,
        format!(
            "altitude floor reached at {:.3} km at {:.2} m/s against a commanded contact speed of \
             {:.1} m/s (admissible [{:.2}, {:.2}]), with {:.1} kg propellant remaining (floor {:.0})",
            l.altitude_km,
            l.descent_rate,
            CONTACT_SPEED_MS,
            lo,
            hi,
            l.propellant,
            PROPELLANT_FLOOR_KG
        ),
    )
}

/// (7) The committed branch's final state re-quantizes inside the bond cap.
///
/// Reads the rank the carrier measured. A branch that reports none fails rather than passing on a
/// substituted cap, which would be a comparison of a constant against itself.
fn gate_compression(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let Some(l) = v.rows().first() else {
        return (false, "no leg witnesses were recorded".into());
    };
    match l.peak_bond {
        Some(bond) => (
            (1..=CAP).contains(&bond),
            format!(
                "the committed branch's final evolved state re-quantizes at peak bond {bond} (cap {CAP})"
            ),
        ),
        None => (
            false,
            "the committed branch reported no rank, so its compression is unmeasured".into(),
        ),
    }
}

fn gate_rebuilds(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let Some(l) = v.rows().first() else {
        return (false, "no leg witnesses were recorded".into());
    };
    (
        l.rebuilds <= MAX_REBUILDS,
        format!(
            "{} carrier rebuild(s) across all legs (cap {MAX_REBUILDS}), read from the pause \
             accessor rather than a log tally. A runaway detector, not a regression band",
            l.rebuilds
        ),
    )
}

fn gate_wall_clock(v: &StudyView<'_, LegSet>) -> (bool, String) {
    let Some(l) = v.rows().first() else {
        return (false, "no leg witnesses were recorded".into());
    };
    (
        l.elapsed_s <= WALL_CLOCK_BUDGET_S,
        format!(
            "{:.1} s elapsed for the whole descent (budget {:.0} s). A runaway detector, not a \
             regression band",
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
pub fn terminal_world(
    steps: usize,
    rho_scale: FloatType,
) -> Result<CompressibleMarchConfig<FloatType>, PhysicsError> {
    world::terminal_descent_world("terminal_descent", measured_atmosphere(rho_scale), steps)
}
