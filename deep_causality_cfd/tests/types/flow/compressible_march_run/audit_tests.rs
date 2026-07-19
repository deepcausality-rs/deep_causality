/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `save_log` audit sink: stepwise flush, a completed run matching its in-memory log, and one
//! file per campaign branch alongside the main spawn/rejoin record.

use super::{EnsRow, field_at_61km, world};
use deep_causality_cfd::{BlackoutTrigger, CfdFlow};

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
