/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `MarchState` is one state behind two transports: a paused march exports it, disk stores it,
//! and a resumed march reads it. The two transports must agree — resuming from an in-memory
//! state and from a saved-then-loaded state produce identical continued reports.

use deep_causality_cfd::{
    Ambient, AtmosphereRow, BlackoutTrigger, CfdFlow, CompressibleMarchConfig,
    CompressibleMarchConfigBuilder, CoupledField, DescentSchedule, MarchState, MarchStop,
    QttObserve, ReferenceScales,
};
use deep_causality_physics::EARTH_RADIUS;
use deep_causality_tensor::Truncation;

const GAMMA_EFF: f64 = 1.1;
const WORLD_FP: &[u8] = b"march-state-test-world-v1";

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

fn world(name: &str, steps: usize) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    CompressibleMarchConfigBuilder::<f64>::new()
        .name(name)
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

/// Pause a march, export its `MarchState`.
fn paused_state() -> MarchState<f64> {
    let cfg = world("nominal_descent", 8);
    let pause = CfdFlow::compressible_march(&cfg)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 3,
        )
        .unwrap();
    pause.state()
}

#[test]
fn a_pause_exports_its_state_with_field_and_step() {
    let state = paused_state();
    assert_eq!(state.step(), 3, "the pause fired at step 3");
    assert!(
        state.field().scalar("truth_state").is_some(),
        "the carried field rides in the state"
    );
}

#[test]
fn the_state_round_trips_through_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("state.dcsnap");
    let state = paused_state();

    state.save(&path, WORLD_FP).expect("saves");
    let loaded = MarchState::<f64>::load(&path, WORLD_FP).expect("loads");

    assert_eq!(loaded.step(), state.step());
    let a = state.field().scalar("truth_state").unwrap();
    let b = loaded.field().scalar("truth_state").unwrap();
    for (x, y) in a.iter().zip(b) {
        assert_eq!(x.to_bits(), y.to_bits(), "carried scalar bit-identical");
    }
}

#[test]
fn disk_resume_equals_in_memory_resume() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("state.dcsnap");
    let state = paused_state();
    state.save(&path, WORLD_FP).expect("saves");
    let loaded = MarchState::<f64>::load(&path, WORLD_FP).expect("loads");

    // Continue the same descent from each transport's field.
    let cfg = world("nominal_descent", 8);
    let from_memory = CfdFlow::compressible_march(&cfg)
        .run_coupled((), state.into_field(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();
    let from_disk = CfdFlow::compressible_march(&cfg)
        .run_coupled((), loaded.into_field(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();

    assert_eq!(
        from_memory.final_field(),
        from_disk.final_field(),
        "resuming from disk is bit-identical to resuming in memory"
    );
    assert_eq!(
        from_memory.series("final_n_tot"),
        from_disk.series("final_n_tot"),
    );
}
