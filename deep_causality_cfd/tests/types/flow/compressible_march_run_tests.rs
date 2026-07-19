/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The compressible coupled host: evolved-state projections, the descent schedule with its
//! shock-fitted inflow strip, rebuild-on-drift, and the shared pause/fork machinery.

use deep_causality_cfd::{
    Ambient, AtmosphereRow, BlackoutTrigger, CfdFlow, CompressibleMarchConfig,
    CompressibleMarchConfigBuilder, CoupledField, DescentSchedule, MarchStop, QttObserve,
    ReferenceScales,
};
use deep_causality_core::AlternatableContext;
use deep_causality_physics::EARTH_RADIUS;
use deep_causality_tensor::Truncation;

const GAMMA_EFF: f64 = 1.1;

fn rows() -> Vec<AtmosphereRow<f64>> {
    vec![
        AtmosphereRow {
            altitude_m: 30_000.0,
            n_tot: 8.0e23,
            temperature: 226.0,
            sound_speed: 301.0,
        },
        AtmosphereRow {
            altitude_m: 61_000.0,
            n_tot: 1.3e21,
            temperature: 250.0,
            sound_speed: 317.0,
        },
        AtmosphereRow {
            altitude_m: 90_000.0,
            n_tot: 7.0e19,
            temperature: 187.0,
            sound_speed: 274.0,
        },
    ]
}

fn reference() -> ReferenceScales<f64> {
    ReferenceScales {
        t_ref: 8_044.0,
        n_ref: 2.645e22,
        u_ref: 376.0,
    }
}

/// A small scheduled descent world: post-shock-like uniform seed, `s_ref` roomy enough that no
/// rebuild triggers.
fn world(name: &str, s_ref: f64, steps: usize) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    CompressibleMarchConfigBuilder::<f64>::new()
        .name(name)
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, s_ref, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(
            QttObserve::default()
                .electron_density()
                .plasma_frequency()
                .blackout_dwell(),
        )
        .schedule(DescentSchedule::new(rows(), GAMMA_EFF).unwrap())
        .reference(reference())
        .build()
        .unwrap()
}

/// A carried field with the truth vehicle at 61 km, flying Mach ~24 tangentially.
fn field_at_61km() -> CoupledField<f64> {
    let mut f = CoupledField::new(Ambient::new(0.01, 0.0, None));
    let r = EARTH_RADIUS + 61_000.0;
    f.set_scalar("truth_state", vec![r, 0.0, 0.0, 0.0, 7_650.0, 0.0]);
    f
}

/// A bare field carrying a flight velocity, so the plume stage can resolve a direction for its
/// along-velocity drag decrement.
fn imprint_field() -> CoupledField<f64> {
    let mut f = CoupledField::new(Ambient::new(0.01, 0.0, None));
    f.set_scalar(
        "truth_state",
        vec![EARTH_RADIUS + 30_000.0, 0.0, 0.0, -400.0, 0.0, 0.0],
    );
    f
}

#[test]
fn coupled_run_publishes_evolved_projections() {
    let cfg = world("evolved", 3.0, 3);
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();

    assert!(pause.error().is_none());
    let field = pause.field();
    let n = 8 * 8;
    for name in ["speed", "T_tr", "n_tot", "pressure_atm"] {
        let s = field
            .scalar(name)
            .unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(s.len(), n, "{name} is per-cell");
        assert!(s.iter().all(|x| x.is_finite()), "{name} finite");
    }
    // The flight scalars follow the truth state through the schedule.
    let alt = field.scalar("flight_altitude").expect("altitude")[0];
    assert!((alt - 61_000.0).abs() < 1.0);
    let mach = field.scalar("flight_mach").expect("mach")[0];
    assert!((mach - 7_650.0 / 317.0).abs() < 1e-9);
}

#[test]
fn inflow_strip_holds_the_rh_post_shock_state() {
    let cfg = world("strip", 3.0, 4);
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();

    // At Mach 24.1 with gamma_eff = 1.1 the RH jump lands T2 near the reference anchor, so the
    // strip's published temperature sits near t_ref (the layer downstream may differ).
    let t_tr = pause.field().scalar("T_tr").expect("T_tr");
    let strip_cell = t_tr[0]; // first column, first row
    assert!(
        (strip_cell - 7_500.0).abs() / 7_500.0 < 0.15,
        "strip holds the post-shock temperature: {strip_cell}"
    );
}

#[test]
fn wave_speed_drift_rebuilds_the_solver_and_logs_it() {
    // A deliberately undersized s_ref: the scheduled inflow's wave speed exceeds it, so the
    // carrier rebuilds and records the rebuild in the provenance log.
    let cfg = world("rebuild", 1.0, 2);
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();

    assert!(pause.error().is_none());
    let log = format!("{}", pause.field().log());
    assert!(log.contains("carrier rebuilt at step 1"), "log: {log}");
}

#[test]
fn world_published_constants_land_on_the_field_each_step() {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let cfg = CompressibleMarchConfigBuilder::<f64>::new()
        .name("commanded")
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 3.0, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(2))
        .observe(QttObserve::default())
        .reference(reference())
        .publish_constant("commanded_bank", 0.35)
        .build()
        .unwrap();
    assert_eq!(cfg.published_constants(), &[("commanded_bank", 0.35)]);

    // No schedule and no truth state: the constant still lands (pre_step publishes it first).
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();
    assert_eq!(pause.field().scalar("commanded_bank"), Some(&[0.35][..]));
}

#[test]
fn without_a_truth_state_the_schedule_is_inert() {
    let cfg = world("inert", 3.0, 2);
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();
    assert!(pause.error().is_none());
    assert!(pause.field().scalar("flight_altitude").is_none());
}

#[test]
fn fork_shares_and_context_alternation_marks_the_branch() {
    let nominal = world("nominal_descent", 3.0, 6);
    let steep = world("steep_descent", 3.0, 6);

    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();
    assert_eq!(pause.step(), 2);

    let fork = pause.fork();
    assert!(fork.shares_fluid_with(&pause), "O(1) fork");

    let branch = pause
        .fork()
        .alternate_context(&steep)
        .continue_march(2)
        .unwrap();
    assert_eq!(branch.name(), "steep_descent");
    let log = format!("{}", branch.effect_log().unwrap());
    assert!(log.contains("!!ContextAlternation!!"), "marker: {log}");
    assert!(branch.final_field().is_some());
}

#[test]
fn continue_branches_matches_the_manual_fork_chain_in_world_order() {
    let nominal = world("nominal_descent", 3.0, 8);
    let shallow = world("shallow_branch", 3.0, 8);
    let steep = world("steep_branch", 3.0, 8);

    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();

    // The fan-out: one report per world, in world order, each with its alternation marker.
    let reports = pause
        .continue_branches(&[&shallow, &steep], 3)
        .expect("both branches complete");
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].name(), "shallow_branch");
    assert_eq!(reports[1].name(), "steep_branch");
    for report in &reports {
        let log = format!("{}", report.effect_log().unwrap());
        assert!(log.contains("!!ContextAlternation!!"), "marker: {log}");
    }

    // Bit-identical to the manual sequential fork of the same pause.
    let manual = pause
        .fork()
        .alternate_context(&steep)
        .continue_march(3)
        .unwrap();
    assert_eq!(reports[1].final_field(), manual.final_field());
    assert_eq!(
        reports[1].series("final_n_tot"),
        manual.series("final_n_tot")
    );
}

#[test]
fn continue_with_matches_the_single_world_batch_and_carries_the_marker() {
    let nominal = world("nominal_descent", 3.0, 8);
    let steep = world("steep_branch", 3.0, 8);

    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();

    // The singular verb: one world, one continued report, marked.
    let single = pause.continue_with(&steep, 3).expect("branch completes");
    assert_eq!(single.name(), "steep_branch");
    let log = format!("{}", single.effect_log().unwrap());
    assert!(log.contains("!!ContextAlternation!!"), "marker: {log}");

    // Bit-identical to the one-world batch fan-out of the same pause.
    let batch = pause.continue_branches(&[&steep], 3).unwrap();
    assert_eq!(single.final_field(), batch[0].final_field());
    assert_eq!(single.series("final_n_tot"), batch[0].series("final_n_tot"));
}

#[test]
fn run_coupled_returns_the_evolved_report() {
    let cfg = world("report", 3.0, 3);
    let report = CfdFlow::march(&cfg)
        .run_coupled((), field_at_61km(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();

    assert_eq!(report.name(), "report");
    // The final field is the evolved translational temperature; the density rides alongside.
    assert_eq!(report.final_field().unwrap().len(), 64);
    assert_eq!(report.series("final_n_tot").unwrap().len(), 64);
    assert_eq!(report.series("final_speed").unwrap().len(), 64);
}

#[test]
fn coupled_report_carries_the_terminal_trajectory_states() {
    let cfg = world("terminal", 3.0, 3);
    let mut field = field_at_61km();
    // A navigation stage would publish this each step; here it is seeded once and carried.
    field.set_scalar("nav_position", vec![6.4e6, 1.0e3, -2.0e3]);
    let report = CfdFlow::march(&cfg)
        .run_coupled((), field, BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();

    let truth = report.series("final_truth_state").expect("truth state");
    assert_eq!(truth.len(), 6, "position + velocity");
    assert!(truth.iter().all(|x| x.is_finite()));
    let nav = report.series("final_nav_position").expect("nav position");
    assert_eq!(nav.len(), 3);

    // Without either witness on the field, the report stays clean.
    let bare = CfdFlow::march(&world("bare", 3.0, 2))
        .run_coupled(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
        )
        .unwrap();
    assert!(bare.series("final_truth_state").is_none());
    assert!(bare.series("final_nav_position").is_none());
}

// ── The campaign event-fork path: `study.fork(&pause).branch(f).continue_for(n)` lowers onto the
// carrier fan-out above, landing on `Marched` so `reduce` reads each branch through a `CaseRun`.

#[derive(Clone)]
struct BranchRow {
    n_tot: f64,
}

impl deep_causality_cfd::TableRow for BranchRow {
    type Scalar = f64;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[("final_n_tot", "-")];
    fn cells(&self) -> Vec<f64> {
        vec![self.n_tot]
    }
}

fn both_branches_reduced(v: &deep_causality_cfd::StudyView<'_, BranchRow>) -> (bool, String) {
    (
        v.rows().len() == 2,
        format!("{} branches reduced", v.rows().len()),
    )
}

#[test]
fn campaign_fork_branch_continue_for_reduces_branches_in_case_order() {
    use deep_causality_cfd::{CaseRun, GateSeq};

    // One shared onset pause; the case axis is the branch command that names each world.
    let nominal = world("nominal_descent", 3.0, 8);
    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();

    let verdict = CfdFlow::study("corridor branches")
        .cases(vec![
            "shallow_branch".to_string(),
            "steep_branch".to_string(),
        ])
        .fork(&pause)
        // `branch` lowers each case onto its own world, built fresh here — a world the study owns,
        // living only in this phase, which the relaxed carrier lifetime now permits.
        .branch(|name| Ok(world(name, 3.0, 8)))
        .continue_for(3)
        .reduce(
            |run: &CaseRun<'_, String, CompressibleMarchConfig<f64>, f64>| {
                // Reports come back in case order: the branch world resumed the case's command.
                assert_eq!(run.report().name(), run.case());
                // Every branch carries the alternation marker the manual fork chain produces.
                let log = format!("{}", run.report().effect_log().unwrap());
                assert!(log.contains("!!ContextAlternation!!"), "marker: {log}");
                let n_tot = run
                    .report()
                    .series("final_n_tot")
                    .map(|s| s[0])
                    .unwrap_or(0.0);
                Ok(BranchRow { n_tot })
            },
        )
        .gates(GateSeq::new("branches").gate("both reduced", both_branches_reduced))
        .verdict()
        .expect("the fork campaign runs to a verdict");

    assert!(verdict.passed(), "{verdict}");
}

fn coarse_round_is_carried(v: &deep_causality_cfd::StudyView<'_, BranchRow>) -> (bool, String) {
    let coarse_ok = v.rounds().len() == 1 && v.rounds()[0].len() == 2;
    let fine_ok = v.rows().len() == 3;
    (
        coarse_ok && fine_ok,
        format!(
            "fine round {} rows over {} prior round(s)",
            v.rows().len(),
            v.rounds().len()
        ),
    )
}

#[test]
fn refine_reforks_the_same_onset_and_carries_the_prior_round() {
    use deep_causality_cfd::{CaseRun, CompressibleMarchConfig, GateSeq};

    let nominal = world("nominal_descent", 3.0, 8);
    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();

    // Reduce each branch to its peak final density (the actual value is immaterial to the test —
    // what matters is that both rounds fork the same onset and the coarse round is retained).
    fn score(
        runs: &[CaseRun<'_, String, CompressibleMarchConfig<f64>, f64>],
    ) -> Result<Vec<BranchRow>, deep_causality_physics::PhysicsError> {
        Ok(runs
            .iter()
            .map(|r| BranchRow {
                n_tot: r
                    .report()
                    .series("final_n_tot")
                    .map(|s| s[0])
                    .unwrap_or(0.0),
            })
            .collect())
    }

    let verdict = CfdFlow::study("two-round corridor")
        .cases(vec!["coarse_a".to_string(), "coarse_b".to_string()])
        .fork(&pause)
        .branch(|name| Ok(world(name, 3.0, 8)))
        .continue_for(3)
        .reduce_all(score)
        // The refinement round re-forks the *same* onset; its three fine cases are derived from
        // the two coarse rows (here just fixed names), and the coarse round is retained.
        .refine(&pause, |coarse: &[BranchRow]| {
            assert_eq!(coarse.len(), 2);
            Ok(vec![
                "fine_a".to_string(),
                "fine_b".to_string(),
                "fine_c".to_string(),
            ])
        })
        .branch(|name| Ok(world(name, 3.0, 8)))
        .continue_for(3)
        .reduce_all(score)
        .gates(GateSeq::new("rounds").gate("coarse carried", coarse_round_is_carried))
        .verdict()
        .expect("the two-round study runs");

    assert!(verdict.passed(), "{verdict}");
}

// ── The origin-fork counterfactual + ensemble path: baseline → alternate → ensemble → couple →
// march_for → reduce_ensemble (the weather campaign's machinery).

struct EnsRow {
    marked: bool,
    draws: usize,
}

fn origin_campaign_is_well_formed(v: &deep_causality_cfd::StudyView<'_, EnsRow>) -> (bool, String) {
    let rows = v.rows();
    // Two cases; the baseline case is unmarked, the alternated case carries the marker; each case
    // flew its full ensemble of draws.
    let ok =
        rows.len() == 2 && !rows[0].marked && rows[1].marked && rows.iter().all(|r| r.draws == 2);
    (
        ok,
        format!(
            "{} cases; baseline marked={}, alternate marked={}; draws {:?}",
            rows.len(),
            rows[0].marked,
            rows[1].marked,
            rows.iter().map(|r| r.draws).collect::<Vec<_>>(),
        ),
    )
}

#[test]
fn origin_campaign_alternates_from_baseline_and_ensembles_the_draws() {
    use deep_causality_cfd::{GateSeq, Report};

    let verdict = CfdFlow::study("origin campaign")
        .cases(vec!["standard".to_string(), "hot".to_string()])
        // The declared baseline built once; `alternate` binds one world per case. The case whose
        // world name matches the baseline ("standard") flies unmarked; "hot" is alternated + marked.
        .baseline(|| Ok(world("standard", 3.0, 4)))
        .alternate(|name| Ok(world(name, 3.0, 4)))
        .ensemble(2) // two receiver-noise draws per case
        .couple(|_case: &String, _draw: usize| ()) // the trivial no-op stack
        .march_for(3, field_at_61km)
        .reduce_ensemble(|_case: &String, draws: &[Report<f64>]| {
            // The ensemble reduction sees the whole draw set per case.
            let marked = draws[0]
                .effect_log()
                .map(|l| format!("{l}").contains("!!ContextAlternation!!"))
                .unwrap_or(false);
            Ok(EnsRow {
                marked,
                draws: draws.len(),
            })
        })
        .gates(GateSeq::new("origin").gate("well formed", origin_campaign_is_well_formed))
        .verdict()
        .expect("the origin campaign runs to a verdict");

    assert!(verdict.passed(), "{verdict}");
}

// ── The save_log audit sink: stepwise flush, completed-run == in-memory log, one file per branch.

#[test]
fn save_log_writes_provenance_that_matches_the_in_memory_log() {
    use std::io::Read;

    let dir = std::env::temp_dir().join("dcl_audit_traj");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("traj.log");
    let _ = std::fs::remove_file(&path);

    // s_ref = 1.0 forces a carrier rebuild each step, so the field's provenance log accrues entries
    // the sink can flush stepwise.
    let cfg = world("audited", 1.0, 4);
    let report = CfdFlow::march(&cfg)
        .save_log(&path)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from_field(field_at_61km())
        .run_for(4)
        .expect("audited run completes");

    let in_memory: Vec<String> = report
        .effect_log()
        .map(|l| l.messages().map(str::to_string).collect())
        .unwrap_or_default();
    assert!(!in_memory.is_empty(), "the rebuild world logs provenance");

    let mut contents = String::new();
    std::fs::File::open(&path)
        .expect("the audit file exists")
        .read_to_string(&mut contents)
        .unwrap();
    let file_lines: Vec<String> = contents.lines().map(str::to_string).collect();
    assert_eq!(
        file_lines, in_memory,
        "the completed run's file equals its in-memory log"
    );
}

#[test]
fn a_run_without_save_log_is_unchanged() {
    // The same run with and without the sink produces the identical in-memory log and report.
    let cfg = world("plain", 1.0, 4);
    let plain = CfdFlow::march(&cfg)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from_field(field_at_61km())
        .run_for(4)
        .unwrap();

    let dir = std::env::temp_dir().join("dcl_audit_unchanged");
    std::fs::create_dir_all(&dir).unwrap();
    let audited = CfdFlow::march(&cfg)
        .save_log(dir.join("p.log"))
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from_field(field_at_61km())
        .run_for(4)
        .unwrap();

    let msgs = |r: &deep_causality_cfd::Report<f64>| -> Vec<String> {
        r.effect_log()
            .map(|l| l.messages().map(str::to_string).collect())
            .unwrap_or_default()
    };
    assert_eq!(
        msgs(&plain),
        msgs(&audited),
        "save_log changes nothing about the run"
    );
    assert_eq!(plain.final_field(), audited.final_field());
}

fn two_rows(v: &deep_causality_cfd::StudyView<'_, EnsRow>) -> (bool, String) {
    (v.rows().len() == 2, format!("{} rows", v.rows().len()))
}

#[test]
fn campaign_save_log_writes_one_file_per_branch_and_a_main_file() {
    use deep_causality_cfd::{GateSeq, Report};

    let dir = std::env::temp_dir().join("dcl_audit_campaign");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let base = dir.join("wx.audit");

    let verdict = CfdFlow::study("audited weather")
        .save_log(&base)
        .cases(vec!["standard".to_string(), "hot".to_string()])
        .baseline(|| Ok(world("standard", 1.0, 4)))
        .alternate(|name| Ok(world(name, 1.0, 4)))
        .ensemble(2)
        .couple(|_c: &String, _d: usize| ())
        .march_for(3, field_at_61km)
        .reduce_ensemble(|_c: &String, draws: &[Report<f64>]| {
            Ok(EnsRow {
                marked: false,
                draws: draws.len(),
            })
        })
        .gates(GateSeq::new("audited").gate("two rows", two_rows))
        .verdict()
        .expect("the audited campaign runs");
    assert!(verdict.passed());

    // 2 cases x 2 draws = 4 per-branch files, each named by case index + world + draw (the case
    // index keeps files unique even if two cases shared a world name), exclusively written.
    for (i, world_name) in ["standard", "hot"].iter().enumerate() {
        for d in 0..2 {
            let p = dir.join(format!(
                "wx.audit.sweep-1.case-{i:02}-{world_name}.draw-{d}.log"
            ));
            assert!(p.exists(), "per-branch file {} exists", p.display());
        }
    }
    // The main file names every spawn and every rejoin outcome.
    let main = std::fs::read_to_string(dir.join("wx.audit.main.log")).expect("main file exists");
    assert!(
        main.contains("fan-out sweep-1"),
        "main names the spawn: {main}"
    );
    assert!(
        main.contains("rejoin sweep-1"),
        "main names the rejoin: {main}"
    );
    assert!(
        main.contains("standard.draw-0"),
        "main names a branch: {main}"
    );
    assert!(main.contains("hot.draw-1"), "main names a branch: {main}");
}

#[test]
fn campaign_save_log_to_an_unwritable_path_fails_the_run() {
    use deep_causality_cfd::{GateSeq, Report};

    // A base whose parent directory does not exist: opening the main audit file fails, and an
    // audited run that can no longer be audited must fail rather than continue silently. The
    // failure surfaces before any marching, so the test is cheap.
    let base = std::path::Path::new("/dcl_no_such_dir_xyz/wx.audit");
    let outcome = CfdFlow::study("bad audit")
        .save_log(base)
        .cases(vec!["a".to_string()])
        .baseline(|| Ok(world("a", 3.0, 3)))
        .alternate(|n| Ok(world(n, 3.0, 3)))
        .ensemble(1)
        .couple(|_c: &String, _d: usize| ())
        .march_for(2, field_at_61km)
        .reduce_ensemble(|_c: &String, _d: &[Report<f64>]| {
            Ok(EnsRow {
                marked: false,
                draws: 1,
            })
        })
        .gates(GateSeq::new("x").gate("two rows", two_rows))
        .verdict();

    let err = outcome.expect_err("an unwritable audit path fails the run");
    assert!(
        format!("{err}").contains("save_log"),
        "the error names the save_log stage: {err}"
    );
}

#[test]
fn origin_campaign_verb_errors_short_circuit_and_name_their_verb() {
    use deep_causality_cfd::{CompressibleMarchConfig, GateSeq, Report};

    // baseline() fails: the whole study short-circuits, tagged with the failing verb.
    let e = CfdFlow::study("x")
        .cases(vec!["a".to_string()])
        .baseline(|| {
            Err::<CompressibleMarchConfig<f64>, _>(
                deep_causality_physics::PhysicsError::CalculationError("no origin".into()),
            )
        })
        .alternate(|n| Ok(world(n, 3.0, 3)))
        .ensemble(1)
        .couple(|_: &String, _: usize| ())
        .march_for(2, field_at_61km)
        .reduce_ensemble(|_: &String, _: &[Report<f64>]| {
            Ok(EnsRow {
                marked: false,
                draws: 1,
            })
        })
        .gates(GateSeq::new("x").gate("two rows", two_rows))
        .verdict()
        .expect_err("a failed baseline short-circuits");
    assert!(format!("{e}").contains("baseline"), "names the verb: {e}");

    // reduce_ensemble() fails after a real march: the error is tagged with reduce_ensemble.
    let e = CfdFlow::study("x")
        .cases(vec!["a".to_string()])
        .baseline(|| Ok(world("a", 3.0, 3)))
        .alternate(|n| Ok(world(n, 3.0, 3)))
        .ensemble(1)
        .couple(|_: &String, _: usize| ())
        .march_for(2, field_at_61km)
        .reduce_ensemble(|_: &String, _: &[Report<f64>]| {
            Err::<EnsRow, _>(deep_causality_physics::PhysicsError::CalculationError(
                "bad row".into(),
            ))
        })
        .gates(GateSeq::new("x").gate("two rows", two_rows))
        .verdict()
        .expect_err("a failed reduction short-circuits");
    assert!(
        format!("{e}").contains("reduce_ensemble"),
        "names the verb: {e}"
    );
}

// ── The de-risk forcing seam (change plasma-retropulsion-de-risk) ───────────────────────────────

/// An unscheduled world (no descent, no inflow strip), optionally imprinting a forcing region.
fn plain_world(
    name: &str,
    steps: usize,
    forcing: Option<deep_causality_cfd::ForcingRegion<f64>>,
) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let mut builder = CompressibleMarchConfigBuilder::<f64>::new()
        .name(name)
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 3.0, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(QttObserve::default())
        .reference(reference());
    if let Some(region) = forcing {
        builder = builder.forcing_region(region);
    }
    builder.build().unwrap()
}

/// A small forcing region on the 8×8 grid of `plain_world`.
fn small_region(target: [f64; 4], eta: f64) -> deep_causality_cfd::ForcingRegion<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let dx = 0.125;
    let mask =
        deep_causality_cfd::plume_mask_2d::<f64>(3, 3, dx, dx, 0.5, 0.5, 0.2, 0.15, dx, &trunc)
            .unwrap();
    deep_causality_cfd::ForcingRegion::new(mask, target, eta).unwrap()
}

#[test]
fn unforced_carrier_matches_the_bare_marcher_bit_for_bit() {
    // The no-forcing bit-identity guarantee: an unscheduled coupled run's final field must equal
    // the bare CompressibleMarcher2d's, component for component, bit for bit — the carrier adds
    // nothing to the march path when no forcing region is configured.
    use deep_causality_cfd::{
        CartesianIdentity, CompressibleMarcher2d, dequantize_2d, quantize_2d,
    };
    use deep_causality_tensor::CausalTensor;

    let steps = 3usize;
    let cfg = plain_world("bare", steps, None);
    let report = CfdFlow::march(&cfg)
        .run_coupled(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
        )
        .unwrap();
    let coupled_t_tr = report.final_field().unwrap().to_vec();

    // The same march, hand-driven on the bare marcher.
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let metric = CartesianIdentity::new(3, 3, 0.125, 0.125, trunc).unwrap();
    let marcher = CompressibleMarcher2d::new(metric, GAMMA_EFF, 0.002, 3.0, trunc).unwrap();
    let n = 8usize;
    // seed_fn(|_,_| (1, 1, 0, 1)) → conserved [ρ, ρu, ρv, E] with p = 1.
    let e0 = 1.0 / (GAMMA_EFF - 1.0) + 0.5;
    let enc = |v: f64| {
        quantize_2d(
            &CausalTensor::new(vec![v; n * n], vec![n, n]).unwrap(),
            &trunc,
        )
        .unwrap()
    };
    let mut state = [enc(1.0), enc(1.0), enc(0.0), enc(e0)];
    for _ in 0..steps {
        state = marcher.step(&state).unwrap();
    }
    // Replicate the carrier's final-field projection: T_tr = (p̂/ρ̂)·t_ref.
    let rho = dequantize_2d(&state[0], 3, 3).unwrap();
    let mx = dequantize_2d(&state[1], 3, 3).unwrap();
    let my = dequantize_2d(&state[2], 3, 3).unwrap();
    let e = dequantize_2d(&state[3], 3, 3).unwrap();
    let bare_t_tr: Vec<f64> = rho
        .as_slice()
        .iter()
        .zip(mx.as_slice())
        .zip(my.as_slice())
        .zip(e.as_slice())
        .map(|(((&r, &a), &b), &en)| {
            let u2 = (a * a + b * b) / (r * r);
            let p_hat = (GAMMA_EFF - 1.0) * (en - 0.5 * r * u2);
            let p_hat = if p_hat > 1.0e-12 { p_hat } else { 1.0e-12 };
            (p_hat / r) * reference().t_ref
        })
        .collect();

    assert_eq!(coupled_t_tr, bare_t_tr, "bit-identical unforced march");
}

#[test]
fn a_forcing_region_changes_the_marched_state() {
    // The same world with and without an imprint diverges — the forcing is live, not decorative.
    let unforced = plain_world("unforced", 3, None);
    let forced = plain_world(
        "forced",
        3,
        Some(small_region([0.5, -0.4, 0.0, 3.0], 0.002)),
    );
    let run = |cfg: &CompressibleMarchConfig<f64>| {
        CfdFlow::march(cfg)
            .run_coupled(
                (),
                CoupledField::new(Ambient::new(0.01, 0.0, None)),
                BlackoutTrigger::new(1.0e9),
                0.0,
            )
            .unwrap()
            .final_field()
            .unwrap()
            .to_vec()
    };
    let a = run(&unforced);
    let b = run(&forced);
    assert_ne!(a, b, "the imprint must change the marched state");
    assert!(b.iter().all(|x| x.is_finite()), "forced state stays finite");
}

#[test]
fn a_branch_world_carries_its_own_forcing_region() {
    // The fork-economics mechanism: continue the same pause into two branch worlds, one with an
    // imprint and one without — the branch rebuild picks up each world's own forcing region, so
    // a per-branch throttle intervention feeds back into that branch's own flow.
    let nominal = plain_world("nominal", 6, None);
    let coast = plain_world("coast_branch", 6, None);
    let burn = plain_world(
        "burn_branch",
        6,
        Some(small_region([0.5, -0.4, 0.0, 3.0], 0.002)),
    );

    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();
    let fork = pause.fork();
    assert!(fork.shares_fluid_with(&pause), "O(1) fork before branching");

    let reports = pause.continue_branches(&[&coast, &burn], 3).unwrap();
    let coast_field = reports[0].final_field().unwrap().to_vec();
    let burn_field = reports[1].final_field().unwrap().to_vec();
    assert_ne!(
        coast_field, burn_field,
        "the branch's own imprint must diverge its flow from the coast branch"
    );
}

#[test]
fn a_forcing_mask_on_the_wrong_grid_is_rejected_at_build() {
    // A mask quantized for a different grid cannot silently ride into the march: the carrier
    // build rejects it before the first step.
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let dx = 1.0 / 16.0;
    // L = 4 mask (8 cores) against the L = 3 grid (6 cores).
    let wrong =
        deep_causality_cfd::plume_mask_2d::<f64>(4, 4, dx, dx, 0.5, 0.5, 0.2, 0.15, dx, &trunc)
            .unwrap();
    let region =
        deep_causality_cfd::ForcingRegion::new(wrong, [0.5, -0.4, 0.0, 3.0], 0.002).unwrap();
    let cfg = plain_world("wrong_grid", 2, Some(region));
    let err = CfdFlow::march(&cfg)
        .run_coupled(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
        )
        .expect_err("mismatched mask must be rejected");
    assert!(
        format!("{err:?}").contains("cores"),
        "names the core-count mismatch: {err:?}"
    );
}

#[test]
fn commanded_throttle_publishes_like_commanded_bank() {
    // The pinned counterfactual seam name for the retropulsion family: a branch world's throttle
    // intervention lands on the field each step through the same publish_constant mechanism.
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let cfg = CompressibleMarchConfigBuilder::<f64>::new()
        .name("throttled")
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 3.0, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(2))
        .observe(QttObserve::default())
        .reference(reference())
        .publish_constant("commanded_throttle", 0.6)
        .build()
        .unwrap();
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();
    assert_eq!(pause.field().scalar("commanded_throttle"), Some(&[0.6][..]));
}

// ── Plume re-imprint: the carrier's field-reading reconfiguration channel (M3) ──

/// A nozzle inside the Cordell validity envelope, matching the retropulsion-stage tests.
fn imprint_nozzle() -> deep_causality_cfd::PlumeNozzle<f64> {
    deep_causality_cfd::PlumeNozzle {
        chamber_pressure_max: 2.0e6,
        chamber_temperature: 1_500.0,
        r_specific: 300.0,
        gamma_jet: 1.3,
        exit_mach: 3.0,
        nozzle_half_angle_rad: 15.0 * std::f64::consts::PI / 180.0,
        throat_diameter: 0.03,
        exit_radius: 0.03407,
        cone_length: 0.0712,
        p_inf: 1_000.0,
        mach_inf: 2.0,
        gamma_inf: 1.4,
    }
}

fn imprint_spec(tolerance: f64, max_refreshes: usize) -> deep_causality_cfd::PlumeImprint<f64> {
    deep_causality_cfd::PlumeImprint {
        throttle_tolerance: tolerance,
        max_refreshes,
        face_x: 0.72,
        axis_y: 0.5,
        smoothing_cells: 1.0,
        domain_m: 4.0,
        target: [1.0, -0.5, 0.0, 2.0],
        eta: 0.002,
    }
}

/// A world that publishes a throttle and opts into plume re-imprint.
fn imprint_world(
    name: &str,
    steps: usize,
    throttle: f64,
    spec: Option<deep_causality_cfd::PlumeImprint<f64>>,
) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let mut builder = CompressibleMarchConfigBuilder::<f64>::new()
        .name(name)
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 3.0, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(QttObserve::default())
        .reference(reference())
        .publish_constant("commanded_throttle", throttle);
    if let Some(s) = spec {
        builder = builder.plume_imprint(s);
    }
    builder.build().unwrap()
}

#[test]
fn the_plume_imprint_follows_the_throttle_through_the_carrier() {
    // End-to-end: PlumeObstruction publishes the geometry into the coupled field; the carrier's
    // pre_step reads it and refreshes the forcing region — the same channel that already carries
    // "truth_state" into the inflow strip. A PhysicsStage never touches the marched layer.
    let cfg = imprint_world("imprint_on", 4, 0.5, Some(imprint_spec(0.01, 8)));
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 2_800.0, 0.785)
        .with_plume_geometry(imprint_nozzle());
    let report = CfdFlow::march(&cfg)
        .run_coupled(stage, imprint_field(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();
    let logged = report
        .effect_log()
        .expect("provenance log")
        .messages()
        .any(|m| m.contains("plume re-imprint"));
    assert!(
        logged,
        "the carrier refreshed the forcing region from the published geometry"
    );
}

#[test]
fn without_the_opt_in_the_carrier_never_re_imprints() {
    // No plume_imprint spec: the forcing region stays exactly as configured at world build, so the
    // march path is untouched and no re-imprint provenance appears.
    let cfg = imprint_world("imprint_off", 4, 0.5, None);
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 2_800.0, 0.785)
        .with_plume_geometry(imprint_nozzle());
    let report = CfdFlow::march(&cfg)
        .run_coupled(stage, imprint_field(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();
    assert!(
        report
            .effect_log()
            .is_none_or(|l| !l.messages().any(|m| m.contains("plume re-imprint"))),
        "no opt-in ⇒ no re-imprint"
    );
}

#[test]
fn a_steady_throttle_re_imprints_once_not_every_step() {
    // The solver-rebuild discipline: with a constant throttle the drift gate fires once and then
    // stays quiet, so a mask rebuild does not happen every step.
    let cfg = imprint_world("imprint_steady", 6, 0.5, Some(imprint_spec(0.01, 8)));
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 2_800.0, 0.785)
        .with_plume_geometry(imprint_nozzle());
    let report = CfdFlow::march(&cfg)
        .run_coupled(stage, imprint_field(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();
    let count = report
        .effect_log()
        .expect("provenance log")
        .messages()
        .filter(|m| m.contains("plume re-imprint"))
        .count();
    assert_eq!(count, 1, "a steady throttle re-imprints exactly once");
}

#[test]
fn the_refresh_cap_bounds_re_imprints() {
    // max_refreshes = 0 forbids any refresh, even with a live throttle and published geometry.
    let cfg = imprint_world("imprint_capped", 4, 0.5, Some(imprint_spec(0.01, 0)));
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 2_800.0, 0.785)
        .with_plume_geometry(imprint_nozzle());
    let report = CfdFlow::march(&cfg)
        .run_coupled(stage, imprint_field(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();
    assert!(
        report
            .effect_log()
            .is_none_or(|l| !l.messages().any(|m| m.contains("plume re-imprint"))),
        "the cap bounds refreshes"
    );
}

// ---------------------------------------------------------------------------
// M4 — leg re-seed visibility and the rebuild budget (terminal-descent-leg)
// ---------------------------------------------------------------------------

/// The `world()` helper with a rebuild budget attached to its schedule.
fn budgeted_world(
    name: &str,
    s_ref: f64,
    steps: usize,
    budget: usize,
) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    CompressibleMarchConfigBuilder::<f64>::new()
        .name(name)
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, s_ref, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .schedule(
            DescentSchedule::new(rows(), GAMMA_EFF)
                .unwrap()
                .with_rebuild_budget(budget),
        )
        .reference(reference())
        .build()
        .unwrap()
}

#[test]
fn the_rebuild_count_is_readable_without_parsing_the_log() {
    // An undersized s_ref forces a rebuild; the count is a number, not a substring tally.
    let cfg = world("rebuild-count", 1.0, 2);
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();

    assert!(pause.rebuilds() >= 1, "the accessor reports the rebuild");
    // And it agrees with what the log says happened.
    let logged = format!("{}", pause.field().log())
        .lines()
        .filter(|l| l.contains("carrier rebuilt at step"))
        .count();
    assert_eq!(pause.rebuilds(), logged);
}

#[test]
fn a_roomy_envelope_reports_no_rebuilds() {
    let cfg = world("no-rebuild", 3.0, 4);
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();
    assert_eq!(pause.rebuilds(), 0);
}

#[test]
fn the_rebuild_budget_is_unbounded_by_default() {
    // The pre-M4 behavior: the hysteresis ratchet bounds the rate, nothing bounds the count.
    let cfg = world("unbounded", 1.0, 4);
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();
    assert!(pause.error().is_none(), "no budget ⇒ no refusal");
}

#[test]
fn exceeding_the_rebuild_budget_refuses() {
    // Budget zero: the very first rebuild the drift demands is refused rather than marched past on
    // a knowingly undersized acoustic envelope.
    let cfg = budgeted_world("budget-zero", 1.0, 4, 0);
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();

    let err = pause.error().expect("the budget refuses");
    assert!(format!("{err:?}").contains("rebuild budget"));
    let log = format!("{}", pause.field().log());
    assert!(log.contains("rebuild budget exhausted"), "log: {log}");
}

#[test]
fn a_budget_above_the_demand_never_fires() {
    let cfg = budgeted_world("budget-roomy", 1.0, 2, 8);
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();
    assert!(pause.error().is_none());
    assert!(pause.rebuilds() <= 8);
}

#[test]
fn a_leg_boundary_records_the_re_seed_in_provenance() {
    // The first leg runs from a fresh field; the second resumes from its MarchState. The re-seed of
    // the marched fluid layer at that boundary must be visible, since the fork path logs its resume
    // and this path previously logged nothing at all.
    let cfg = world("leg-one", 3.0, 2);
    let first = CfdFlow::march(&cfg)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from_field(field_at_61km())
        .until(|_, _| false)
        .unwrap();

    let before = format!("{}", first.field().log());
    assert!(
        !before.contains("leg re-seeded"),
        "a fresh march is not a re-seed: {before}"
    );

    let next = world("leg-two", 3.0, 2);
    let second = CfdFlow::march(&next)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from(first.state())
        .until(|_, _| false)
        .unwrap();

    let log = format!("{}", second.field().log());
    assert!(log.contains("leg re-seeded"), "log: {log}");
    assert!(
        log.contains("leg-two"),
        "the entry names the incoming world: {log}"
    );
}

#[test]
fn the_re_seed_entry_leaves_the_existing_message_texts_alone() {
    // Downstream gates match on these exact substrings; the new entry must not perturb them.
    let cfg = world("texts-one", 1.0, 2);
    let first = CfdFlow::march(&cfg)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from_field(field_at_61km())
        .until(|_, _| false)
        .unwrap();
    let log = format!("{}", first.field().log());
    assert!(log.contains("carrier rebuilt at step"), "log: {log}");
}

#[test]
fn every_continued_branch_records_what_its_fork_cost() {
    let nominal = world("nominal_descent", 3.0, 6);
    let steep = world("steep_descent", 3.0, 6);

    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();

    // The `continue_with` path — what the study grammar's `branch` lowers onto, and the one that
    // never builds a `CarrierFork`, so the record is the only way a study can see the economics.
    let branch = pause.continue_with(&steep, 2).unwrap();
    let e = branch
        .fork_economics()
        .expect("a continued branch must record what its fork cost");
    assert!(e.shares_fluid(), "the branch must enter by reference");
    assert!(e.shares_field(), "the coupled field too");
    assert!(
        e.fluid_refs() > 1,
        "a share, not sole ownership: the pause still holds its own reference"
    );
    assert!(e.is_o1());

    // The manual fork chain records the same facts.
    let forked = pause
        .fork()
        .alternate_context(&steep)
        .continue_march(2)
        .unwrap();
    assert_eq!(forked.fork_economics().map(|e| e.is_o1()), Some(true));

    // A plain march forked nothing and must not claim it did.
    let plain = CfdFlow::march(&nominal)
        .run_coupled((), field_at_61km(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();
    assert!(plain.fork_economics().is_none());
}

#[test]
fn a_fan_out_shares_one_paused_state_across_every_branch() {
    let nominal = world("nominal_descent", 3.0, 6);
    let a = world("branch_a", 3.0, 6);
    let b = world("branch_b", 3.0, 6);
    let c = world("branch_c", 3.0, 6);

    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();

    let reports = pause.continue_branches(&[&a, &b, &c], 2).unwrap();

    assert_eq!(reports.len(), 3);
    for r in &reports {
        let e = r
            .fork_economics()
            .expect("fan-out branches record economics");
        assert!(
            e.is_o1(),
            "a roster of N must cost one paused state, not N copies: {} was not an O(1) fork",
            r.name()
        );
    }
}
