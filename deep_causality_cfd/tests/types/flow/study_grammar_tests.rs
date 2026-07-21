/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The campaign grammar, pointwise path: `CfdFlow::study(..).cases(..).sweep(..).record(..)
//! .gates(..).verdict()`. The happy path passes, a failing gate fails the verdict without losing
//! the recorded table, a sweep error short-circuits and names the verb, and a second gating
//! sequence merges into one verdict.

use deep_causality_cfd::{
    CaseRun, CfdFlow, DuctAreaProfile, DuctConfig, DuctInlet, DuctStop, EvidenceClass, GateSeq,
    StudyView,
};
use deep_causality_file::{FromTableRow, TableRow};
use deep_causality_physics::PhysicsError;

#[derive(Debug, Clone, PartialEq)]
struct MapRow {
    p: f64,
    cf: f64,
}

impl TableRow for MapRow {
    type Scalar = f64;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[("p", "-"), ("cf", "-")];
    fn cells(&self) -> Vec<f64> {
        vec![self.p, self.cf]
    }
}

/// One case, one row: cf grows with the back-pressure ratio.
fn map_row(p: &f64) -> Result<MapRow, PhysicsError> {
    Ok(MapRow {
        p: *p,
        cf: 1.0 + *p,
    })
}

/// A case body that diverges on a sentinel value.
fn map_row_or_fail(p: &f64) -> Result<MapRow, PhysicsError> {
    if *p < 0.0 {
        Err(PhysicsError::CalculationError(
            "negative back pressure".into(),
        ))
    } else {
        Ok(MapRow {
            p: *p,
            cf: 1.0 + *p,
        })
    }
}

fn gate_positive_cf(v: &StudyView<'_, MapRow>) -> (bool, String) {
    let ok = v.rows().iter().all(|r| r.cf > 0.0);
    (ok, format!("{} rows, all cf > 0", v.rows().len()))
}

fn gate_every_case_reduced(v: &StudyView<'_, MapRow>) -> (bool, String) {
    let ok = v.rows().len() == v.cases_len();
    (
        ok,
        format!("{} rows for {} cases", v.rows().len(), v.cases_len()),
    )
}

fn gate_always_fails(_: &StudyView<'_, MapRow>) -> (bool, String) {
    (false, "forced regression".to_string())
}

fn map_gates() -> GateSeq<MapRow> {
    GateSeq::new("nozzle map")
        .gate("physical thrust", gate_positive_cf)
        .gate("every case reduced", gate_every_case_reduced)
}

#[test]
fn pointwise_study_passes_all_gates_and_records_the_table() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("map.csv");

    let verdict = CfdFlow::study("nozzle map")
        .cases(vec![0.9_f64, 0.6, 0.3])
        .sweep(map_row)
        .record(&path)
        .gates(map_gates())
        .verdict()
        .expect("no setup error");

    assert!(verdict.passed(), "{verdict}");
    assert!(path.exists(), "the table was recorded");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.starts_with("p,cf\n#units,-,-\n"),
        "schema header: {text}"
    );
}

#[test]
fn a_failing_gate_fails_the_verdict_but_keeps_the_recorded_table() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("map.csv");

    let verdict = CfdFlow::study("nozzle map")
        .cases(vec![0.9_f64])
        .sweep(map_row)
        .record(&path)
        .gates(GateSeq::new("strict").gate("forced fail", gate_always_fails))
        .verdict()
        .expect("no setup error");

    assert!(!verdict.passed(), "the forced gate fails the verdict");
    assert!(
        path.exists(),
        "recording precedes judgment, so the table survives"
    );
}

#[test]
fn a_sweep_error_short_circuits_and_names_the_verb() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("map.csv");

    let outcome = CfdFlow::study("nozzle map")
        .cases(vec![0.9_f64, -1.0, 0.3])
        .sweep(map_row_or_fail)
        .record(&path)
        .gates(map_gates())
        .verdict();

    let err = outcome.expect_err("the diverging case short-circuits the study");
    assert_eq!(err.stage(), "sweep", "the error names the verb: {err}");
    assert!(!path.exists(), "a short-circuited study records nothing");
}

// ── The march path: case -> march -> reduce ───────────────────────────────────────────────────

/// A converging-diverging nozzle case per back-pressure ratio.
fn duct_case(p_ratio: &f64) -> Result<DuctConfig<f64>, PhysicsError> {
    let p0 = 101_325.0;
    DuctConfig::new(
        DuctAreaProfile::ConvergingDiverging {
            inlet_area: 2.0,
            throat_area: 1.0,
            exit_area: 2.0,
            length: 1.0,
        },
        DuctInlet { p0, t0: 300.0 },
        1.4,
        p0 * p_ratio,
        64,
        DuctStop {
            max_steps: 2_000,
            residual_tol: 1.0e-8,
        },
    )
}

/// Reduce a duct report to a map row: the case ratio and the thrust coefficient.
fn map_row_from_run(run: &CaseRun<'_, f64, DuctConfig<f64>, f64>) -> Result<MapRow, PhysicsError> {
    let cf = run
        .report()
        .series("thrust_coefficient")
        .and_then(|s| s.first().copied())
        .ok_or_else(|| PhysicsError::CalculationError("no thrust coefficient".into()))?;
    Ok(MapRow { p: *run.case(), cf })
}

// ── Entry verbs: read a schedule, read a matrix + prepare a rig ────────────────────────────────

#[derive(Debug, Clone)]
struct FlightPoint {
    mach: f64,
    alt_km: f64,
}

impl TableRow for FlightPoint {
    type Scalar = f64;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[("mach", "-"), ("alt", "km")];
    fn cells(&self) -> Vec<f64> {
        vec![self.mach, self.alt_km]
    }
}

impl FromTableRow for FlightPoint {
    fn from_cells(cells: &[f64]) -> Option<Self> {
        Some(Self {
            mach: cells[0],
            alt_km: cells[1],
        })
    }
}

#[test]
fn read_entry_loads_a_schedule_column_as_the_case_axis() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("schedule.csv");
    std::fs::write(&path, "p\n#units,-\n0.9\n0.6\n0.3\n").unwrap();

    let verdict = CfdFlow::study("nozzle")
        .read::<f64>(&path, "p")
        .sweep(map_row)
        .gates(map_gates())
        .verdict()
        .expect("no setup error");

    assert!(verdict.passed(), "{verdict}");
}

#[test]
fn matrix_entry_and_prepared_rig_share_the_apparatus() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("matrix.csv");
    std::fs::write(&path, "mach,alt\n#units,-,km\n1.2,11.0\n0.8,0.0\n").unwrap();

    let verdict = CfdFlow::study("placard")
        .matrix::<FlightPoint>(&path)
        // The rig: a shared bias every case reads by reference.
        .prepare(|| Ok(0.5_f64))
        .sweep(|rig: &f64, fp: &FlightPoint| {
            Ok(MapRow {
                p: fp.mach,
                cf: *rig + fp.alt_km + 1.0,
            })
        })
        .gates(map_gates())
        .verdict()
        .expect("no setup error");

    assert!(verdict.passed(), "{verdict}");
}

#[test]
fn march_path_study_reduces_reports_to_rows() {
    let verdict = CfdFlow::study("nozzle operating map")
        .cases(vec![0.5_f64, 0.3])
        .case(duct_case)
        .march::<f64>()
        .reduce(map_row_from_run)
        .gates(map_gates())
        .verdict()
        .expect("no setup error");

    assert!(verdict.passed(), "{verdict}");
}

#[test]
fn reduce_all_sees_every_run_for_a_cross_case_reference() {
    // Score every case against a reference derived from the first case's run (the aim-point
    // pattern): reduce_all sees all runs at once.
    let verdict = CfdFlow::study("nozzle operating map")
        .cases(vec![0.5_f64, 0.3])
        .case(duct_case)
        .march::<f64>()
        .reduce_all(|runs: &[CaseRun<'_, f64, DuctConfig<f64>, f64>]| {
            let reference = runs[0]
                .report()
                .series("thrust_coefficient")
                .and_then(|s| s.first().copied())
                .ok_or_else(|| PhysicsError::CalculationError("no reference".into()))?;
            // One row per case, in order, each scored against the shared reference.
            Ok(runs
                .iter()
                .map(|r| MapRow {
                    p: *r.case(),
                    cf: reference,
                })
                .collect())
        })
        .gates(map_gates())
        .verdict()
        .expect("no setup error");

    assert!(verdict.passed(), "{verdict}");
}

#[test]
fn a_second_gating_sequence_merges_into_one_verdict() {
    let verdict = CfdFlow::study("nozzle map")
        .cases(vec![0.9_f64, 0.6])
        .sweep(map_row)
        .gates(GateSeq::new("A").gate("thrust", gate_positive_cf))
        .gates(GateSeq::new("B").gate("count", gate_every_case_reduced))
        .verdict()
        .expect("no setup error");

    assert!(verdict.passed());
    assert_eq!(
        verdict.outcomes().len(),
        2,
        "both sequences' gates are present"
    );
}

#[test]
fn verdict_renders_merges_and_exposes_outcomes_via_studyview_of() {
    // A bespoke row set checked outside a campaign phase — the trajectory-gate pattern
    // (StudyView::of), exercising the Verdict's rendering, merge, and accessors directly.
    let rows = vec![MapRow { p: 0.5, cf: 1.5 }, MapRow { p: 0.9, cf: 1.9 }];
    let view = StudyView::of(&rows);
    assert_eq!(view.rows().len(), 2);
    assert_eq!(view.cases_len(), 2);
    assert!(view.rounds().is_empty());
    assert_eq!(view.title(), "");

    // A passing verdict: title, outcomes, GateOutcome accessors, no warnings, PASS rendering.
    let pass = map_gates().check(&view);
    assert!(pass.passed());
    assert_eq!(pass.title(), "nozzle map");
    assert_eq!(pass.outcomes().len(), 2);
    assert_eq!(pass.outcomes()[0].label(), "physical thrust");
    assert!(pass.outcomes()[0].passed());
    assert!(pass.outcomes()[0].detail().contains("cf > 0"));
    assert!(pass.warnings().is_empty());
    // `gate` records a tripwire: a bound with no declared provenance cannot claim to be a
    // reference, so the rendered line carries the weaker class.
    assert_eq!(pass.outcomes()[0].evidence(), EvidenceClass::Tripwire);
    let rendered = format!("{pass}");
    assert!(
        rendered.contains("[PASS] [tripwire] physical thrust:"),
        "{rendered}"
    );
    assert!(rendered.contains("=== All gates passed"), "{rendered}");

    // A failing verdict renders the FAIL line and the regression footer.
    let fail = GateSeq::new("regression")
        .gate("boom", gate_always_fails)
        .check(&view);
    assert!(!fail.passed());
    let fr = format!("{fail}");
    assert!(
        fr.contains("[FAIL] [tripwire] boom: forced regression"),
        "{fr}"
    );
    assert!(fr.contains("REGRESSION"), "{fr}");

    // `reference_gate` marks a bound backed by an analytic or published value.
    let referenced = GateSeq::new("referenced")
        .reference_gate("physical thrust", gate_positive_cf)
        .check(&view);
    assert_eq!(
        referenced.outcomes()[0].evidence(),
        EvidenceClass::Reference
    );
    let rr = format!("{referenced}");
    assert!(rr.contains("[PASS] [reference] physical thrust:"), "{rr}");

    // merge: titles join, outcomes concatenate, passed is the conjunction.
    let merged = pass.merge(fail);
    assert!(!merged.passed());
    assert_eq!(merged.outcomes().len(), 3);
    assert_eq!(merged.title(), "nozzle map + regression");
}
