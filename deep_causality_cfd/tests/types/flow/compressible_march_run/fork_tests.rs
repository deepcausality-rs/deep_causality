/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The pause/fork machinery: O(1) state sharing, the fan-out and its singular sibling, the
//! terminal trajectory witnesses a report carries, and the fork-economics record.

use super::{field_at_61km, world};
use deep_causality_cfd::{Ambient, BlackoutTrigger, CfdFlow, CoupledField};
use deep_causality_core::AlternatableContext;

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
