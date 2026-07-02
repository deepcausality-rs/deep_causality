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

#[test]
fn coupled_run_publishes_evolved_projections() {
    let cfg = world("evolved", 3.0, 3);
    let pause = CfdFlow::compressible_march(&cfg)
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
    let pause = CfdFlow::compressible_march(&cfg)
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
    let pause = CfdFlow::compressible_march(&cfg)
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
    let pause = CfdFlow::compressible_march(&cfg)
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
    let pause = CfdFlow::compressible_march(&cfg)
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

    let pause = CfdFlow::compressible_march(&nominal)
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
fn run_coupled_returns_the_evolved_report() {
    let cfg = world("report", 3.0, 3);
    let report = CfdFlow::compressible_march(&cfg)
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
    let report = CfdFlow::compressible_march(&cfg)
        .run_coupled((), field, BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();

    let truth = report.series("final_truth_state").expect("truth state");
    assert_eq!(truth.len(), 6, "position + velocity");
    assert!(truth.iter().all(|x| x.is_finite()));
    let nav = report.series("final_nav_position").expect("nav position");
    assert_eq!(nav.len(), 3);

    // Without either witness on the field, the report stays clean.
    let bare = CfdFlow::compressible_march(&world("bare", 3.0, 2))
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
