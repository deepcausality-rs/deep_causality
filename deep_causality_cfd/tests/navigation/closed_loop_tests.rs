/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The closed-loop navigation gate (decision ⑦): drive the `ReentryNavEngine` with a synthetic
//! bias-corrupted `ImuModel` against a KS ground-truth orbit through a GNSS-blackout window. The
//! load-bearing behaviour — (a) GNSS-aided tracking is tight, (b) INS-only error grows through the
//! blackout, (c) the returning GNSS fix reacquires — and the through-plasma optical aid bounds the
//! blackout drift.

use deep_causality_cfd::{ImuModel, InsErrorState, NavFilter, ReentryNavEngine};
use deep_causality_physics::{EARTH_GM, KsPropagator};

fn setup() -> (ReentryNavEngine<f64>, KsPropagator<f64>, ImuModel<f64>) {
    let (r0, v0) = ([7.0e6, 1.0e6, 2.0e6], [-1.0e3, 6.5e3, 3.0e3]);
    let truth = KsPropagator::from_state(r0, v0, EARTH_GM).unwrap();
    let filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]).unwrap();
    let eng = ReentryNavEngine::new(r0, v0, EARTH_GM, filter);
    // ~2 cm/s² accelerometer bias — the drift source; nav-grade process noise.
    let imu = ImuModel::new([2.0e-2, 0.0, 0.0], [0.0; 3], [1.0e-6; 17]);
    (eng, truth, imu)
}

fn err(eng: &ReentryNavEngine<f64>, truth: &KsPropagator<f64>) -> f64 {
    let (rt, _) = truth.propagate(eng.elapsed_time()).unwrap();
    let p = eng.position();
    ((p[0] - rt[0]).powi(2) + (p[1] - rt[1]).powi(2) + (p[2] - rt[2]).powi(2)).sqrt()
}

#[test]
fn closed_loop_tracks_then_drifts_then_reacquires() {
    let (mut eng, truth, imu) = setup();
    let (dt, force) = (1.0, [0.0; 3]); // coasting truth; the IMU adds the bias

    // Phase 1 — GNSS available (pre-blackout): a fix each step keeps the nominal on truth.
    for _ in 0..30 {
        eng.predict(
            dt,
            imu.sense_specific_force(force),
            imu.sense_angular_rate([0.0; 3]),
            imu.process_noise(),
        )
        .unwrap();
        let (rt, _) = truth.propagate(eng.elapsed_time()).unwrap();
        eng.correct_position(rt, 0.01).unwrap(); // GNSS: 0.1 m 1σ
    }
    let err_pre = err(&eng, &truth);

    // Phase 2 — blackout: GNSS denied (physics-gated by the ④ flag), INS-only dead-reckoning.
    for _ in 0..60 {
        eng.predict(
            dt,
            imu.sense_specific_force(force),
            imu.sense_angular_rate([0.0; 3]),
            imu.process_noise(),
        )
        .unwrap();
    }
    let err_blackout = err(&eng, &truth);

    // Phase 3 — GNSS reacquired.
    for _ in 0..30 {
        eng.predict(
            dt,
            imu.sense_specific_force(force),
            imu.sense_angular_rate([0.0; 3]),
            imu.process_noise(),
        )
        .unwrap();
        let (rt, _) = truth.propagate(eng.elapsed_time()).unwrap();
        eng.correct_position(rt, 0.01).unwrap();
    }
    let err_post = err(&eng, &truth);

    assert!(err_pre < 1.0, "GNSS-aided tracking is tight: {err_pre} m");
    assert!(
        err_blackout > 10.0 * err_pre.max(1e-3),
        "INS-only error grows through the blackout: pre {err_pre} m -> blackout {err_blackout} m"
    );
    assert!(
        err_post < 0.2 * err_blackout,
        "reacquisition re-converges: blackout {err_blackout} m -> post {err_post} m"
    );
}

#[test]
fn through_plasma_optical_aid_bounds_the_blackout_drift() {
    // Unaided blackout: INS-only for 200 s.
    let (mut e1, truth1, imu1) = setup();
    let mut max_unaided = 0.0f64;
    for _ in 0..200 {
        e1.predict(
            1.0,
            imu1.sense_specific_force([0.0; 3]),
            imu1.sense_angular_rate([0.0; 3]),
            imu1.process_noise(),
        )
        .unwrap();
        max_unaided = max_unaided.max(err(&e1, &truth1));
    }

    // Aided blackout: a coarse through-plasma optical fix (~50 m 1σ) every 20 s.
    let (mut e2, truth2, imu2) = setup();
    let mut max_aided = 0.0f64;
    for k in 1..=200 {
        e2.predict(
            1.0,
            imu2.sense_specific_force([0.0; 3]),
            imu2.sense_angular_rate([0.0; 3]),
            imu2.process_noise(),
        )
        .unwrap();
        if k % 20 == 0 {
            let (rt, _) = truth2.propagate(e2.elapsed_time()).unwrap();
            e2.correct_position(rt, 2500.0).unwrap(); // optical: 50 m 1σ (variance 2500)
        }
        max_aided = max_aided.max(err(&e2, &truth2));
    }

    assert!(
        max_aided < 0.5 * max_unaided,
        "the through-plasma optical aid bounds the drift: aided {max_aided} m vs unaided {max_unaided} m"
    );
}
