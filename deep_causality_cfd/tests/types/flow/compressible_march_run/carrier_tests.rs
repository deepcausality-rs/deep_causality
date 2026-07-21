/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The carrier itself: evolved projections, the shock-fitted inflow strip, world constants, the
//! rebuild accounting and its budget, and leg-re-seed provenance.

use super::{GAMMA_EFF, budgeted_world, field_at_61km, reference, world};
use deep_causality_cfd::{
    Ambient, BlackoutTrigger, CfdFlow, CompressibleMarchConfigBuilder, CoupledField, MarchStop,
    QttObserve,
};
use deep_causality_tensor::Truncation;

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
