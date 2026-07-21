/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The named-stage coupled march builder (`.couple(..).from(..).until/run/run_for`) must be a
//! faithful, readable rewrite of the positional `run_until` / `run_coupled`: same coupling, same
//! field, same trigger and kappa give the same result.

use deep_causality_cfd::{
    Ambient, AtmosphereRow, BlackoutTrigger, CfdFlow, CompressibleMarchConfig,
    CompressibleMarchConfigBuilder, CoupledField, DescentSchedule, LEG_RE_SEEDS_FIELD, MarchState,
    MarchStop, QttObserve, ReferenceScales,
};
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

fn world(steps: usize) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    CompressibleMarchConfigBuilder::<f64>::new()
        .name("nominal_descent")
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 3.0, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(QttObserve::default().electron_density())
        .schedule(DescentSchedule::new(rows(), GAMMA_EFF).unwrap())
        .reference(ReferenceScales {
            t_ref: 8_044.0,
            n_ref: 2.645e22,
            u_ref: 376.0,
        })
        .build()
        .unwrap()
}

fn field_at_61km() -> CoupledField<f64> {
    let mut f = CoupledField::new(Ambient::new(0.01, 0.0, None));
    let r = EARTH_RADIUS + 61_000.0;
    f.set_scalar("truth_state", vec![r, 0.0, 0.0, 0.0, 7_650.0, 0.0]);
    f
}

#[test]
fn builder_run_matches_run_coupled() {
    let cfg = world(4);
    let trigger = BlackoutTrigger::new(1.0e9);

    let via_builder = CfdFlow::march(&cfg)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from(MarchState::new(field_at_61km()))
        .run()
        .unwrap();

    let via_positional = CfdFlow::march(&cfg)
        .run_coupled((), field_at_61km(), trigger, 0.0)
        .unwrap();

    assert_eq!(via_builder.final_field(), via_positional.final_field());
    assert_eq!(
        via_builder.series("final_n_tot"),
        via_positional.series("final_n_tot")
    );
}

#[test]
fn builder_run_for_overrides_the_horizon_and_matches_march_with() {
    // The config horizon is 4; run_for(2) must override it.
    let cfg = world(4);
    let trigger = BlackoutTrigger::new(1.0e9);

    let via_builder = CfdFlow::march(&cfg)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from(MarchState::new(field_at_61km()))
        .run_for(2)
        .unwrap();

    let via_positional = CfdFlow::march(&cfg)
        .march_with(MarchStop::Fixed(2))
        .run_coupled((), field_at_61km(), trigger, 0.0)
        .unwrap();

    assert_eq!(via_builder.final_field(), via_positional.final_field());
}

#[test]
fn builder_until_pauses_at_the_event() {
    let cfg = world(8);
    let pause = CfdFlow::march(&cfg)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from(MarchState::new(field_at_61km()))
        .until(|_, s| s >= 3)
        .unwrap();

    // The named `until` wires to run_until: it pauses at the predicate and carries the field.
    assert_eq!(pause.step(), 3);
    assert!(pause.field().scalar("truth_state").is_some());
    // And the exported state resumes exactly this pause.
    assert_eq!(pause.state().step(), 3);
}

// ── The typed leg re-seed counter (change `fix-retropulsion-measurement-integrity`) ───────────

#[test]
fn the_re_seed_counter_accumulates_across_chained_legs() {
    // `from` re-seeds the marched layer from the world's seed and records it. The count is carried
    // on the coupled field, so it accumulates across legs — which is what a consumer asking how many
    // leg boundaries a descent crossed wants. Counting "leg re-seeded" substrings in a rendered log
    // answers the same question only for as long as the message keeps its wording.
    let cfg = world(8);

    // `from_field` is a fresh march: nothing was re-seeded.
    let first = CfdFlow::march(&cfg)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from_field(field_at_61km())
        .until(|_, s| s >= 2)
        .unwrap();
    assert_eq!(first.re_seeds(), 0.0);
    assert!(first.field().scalar(LEG_RE_SEEDS_FIELD).is_none());

    // Each chained leg adds one.
    let second = CfdFlow::march(&cfg)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from(first.state())
        .until(|_, s| s >= 2)
        .unwrap();
    assert_eq!(second.re_seeds(), 1.0);

    let third = CfdFlow::march(&cfg)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from(second.state())
        .until(|_, s| s >= 2)
        .unwrap();
    assert_eq!(third.re_seeds(), 2.0);
}

#[test]
fn the_re_seed_counter_matches_the_logged_entries() {
    // The counter replaces counting the prose, so the two must agree.
    let cfg = world(8);
    let mut pause = CfdFlow::march(&cfg)
        .couple(())
        .trigger(BlackoutTrigger::new(1.0e9))
        .from_field(field_at_61km())
        .until(|_, s| s >= 2)
        .unwrap();
    for _ in 0..3 {
        pause = CfdFlow::march(&cfg)
            .couple(())
            .trigger(BlackoutTrigger::new(1.0e9))
            .from(pause.state())
            .until(|_, s| s >= 2)
            .unwrap();
    }
    let logged = pause
        .field()
        .log()
        .messages()
        .filter(|m| m.contains("leg re-seeded"))
        .count();
    assert_eq!(pause.re_seeds(), logged as f64);
    assert_eq!(logged, 3);
}
