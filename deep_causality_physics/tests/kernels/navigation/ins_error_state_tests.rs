/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The strapdown-INS error state (`InsErrorState`) — the load-bearing GNSS-denial drift laws: a constant
//! accelerometer bias grows the position error as `t²`, a constant gyro bias as `t³`, and the carried
//! clock error accumulates linearly. These are the growth rates the closed-loop navigation gate reads.

use deep_causality_physics::InsErrorState;

/// March the error state for `steps` of `dt` under a constant specific force; return the position-error
/// norm at the end.
fn drift_norm(state: InsErrorState<f64>, dt: f64, steps: usize, f: [f64; 3]) -> f64 {
    let mut s = state;
    for _ in 0..steps {
        s = s.propagate(dt, f);
    }
    s.position_error_norm()
}

#[test]
fn accelerometer_bias_grows_position_error_as_t_squared() {
    // b_a only, zero specific force ⇒ δp = ½·b_a·t². Doubling the horizon quadruples the drift.
    let state = InsErrorState::from_biases([1e-3, 0.0, 0.0], [0.0; 3]);
    let dt = 0.01;
    let d_t = drift_norm(state, dt, 1000, [0.0; 3]); // horizon T
    let d_2t = drift_norm(state, dt, 2000, [0.0; 3]); // horizon 2T
    let ratio = d_2t / d_t;
    assert!(
        (ratio - 4.0).abs() < 0.05,
        "accel-bias drift should scale as t² (ratio ≈ 4): {ratio}"
    );
}

#[test]
fn gyro_bias_grows_position_error_as_t_cubed() {
    // b_g only, with a non-zero specific force f (the gyro tilt mis-projects f) ⇒ δp ∝ t³.
    let state = InsErrorState::from_biases([0.0; 3], [0.0, 0.0, 1e-4]);
    let f = [9.81, 0.0, 0.0]; // specific-force reaction
    let dt = 0.01;
    let d_t = drift_norm(state, dt, 1000, f);
    let d_2t = drift_norm(state, dt, 2000, f);
    let ratio = d_2t / d_t;
    assert!(
        (ratio - 8.0).abs() < 0.2,
        "gyro-bias drift should scale as t³ (ratio ≈ 8): {ratio}"
    );
}

#[test]
fn carried_clock_error_accumulates_linearly() {
    // A clock drift with zero initial bias ⇒ bias grows linearly in time (doubling ⇒ ×2).
    let state = InsErrorState::<f64>::zero().with_clock(0.0, 2.5e-9);
    let dt = 0.01;
    let mut s = state;
    for _ in 0..1000 {
        s = s.propagate(dt, [0.0; 3]);
    }
    let b_t = s.clock_bias();
    for _ in 0..1000 {
        s = s.propagate(dt, [0.0; 3]);
    }
    let b_2t = s.clock_bias();
    assert!((b_t - 2.5e-9 * 10.0).abs() < 1e-18, "bias = drift·t: {b_t}");
    assert!(
        (b_2t / b_t - 2.0).abs() < 1e-9,
        "linear accumulation: {}",
        b_2t / b_t
    );
    // Drift is unchanged by the deterministic step.
    assert!((s.clock_drift() - 2.5e-9).abs() < 1e-24);
}

#[test]
fn zero_error_state_never_drifts() {
    let s = InsErrorState::<f64>::zero();
    let out = drift_norm(s, 0.01, 5000, [9.81, 0.0, 0.0]);
    assert!(
        out < 1e-12,
        "a perfectly-known nominal does not drift: {out}"
    );
}

#[test]
fn getters_round_trip_the_error_state() {
    let s = InsErrorState::from_biases([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]).with_clock(7.0, 8.0);
    assert_eq!(s.accel_bias(), [1.0, 2.0, 3.0]);
    assert_eq!(s.gyro_bias(), [4.0, 5.0, 6.0]);
    assert_eq!(s.clock_bias(), 7.0);
    assert_eq!(s.clock_drift(), 8.0);
    assert_eq!(s.position_error(), [0.0; 3]);
    assert_eq!(s.velocity_error(), [0.0; 3]);
    assert_eq!(s.attitude_error(), [0.0; 3]);
}
