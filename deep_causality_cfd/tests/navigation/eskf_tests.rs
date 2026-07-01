/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The 17-state ESKF (`NavFilter`): the linearised transition matrix is consistent with the state
//! propagation, predict grows the position uncertainty (dead-reckoning through blackout), a measurement
//! shrinks it and pulls the estimate toward the fix, and the closed loop reacquires — the load-bearing
//! filter behaviour the navigation gate rests on.

use deep_causality_cfd::{InsErrorState, NavFilter, nav_transition_matrix};

#[test]
fn f_matrix_matches_propagate() {
    // F·x must equal the state propagation of x (one consistent linearisation for state + covariance).
    let dt = 0.02;
    let f = [3.0, -1.0, 9.81];
    let x =
        InsErrorState::from_biases([0.1, -0.2, 0.05], [1e-3, 2e-3, -1e-3]).with_clock(0.3, 4e-9);
    let mat = nav_transition_matrix(dt, f);
    let xv = x.to_array();
    let fx: [f64; 17] = core::array::from_fn(|i| (0..17).fold(0.0, |s, j| s + mat[i][j] * xv[j]));
    let prop = x.propagate(dt, f).to_array();
    let max = fx
        .iter()
        .zip(prop.iter())
        .fold(0.0f64, |m, (a, b)| m.max((a - b).abs()));
    assert!(max < 1e-15, "F·x must equal propagate(x): max diff {max}");
}

#[test]
fn predict_grows_position_uncertainty() {
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]);
    let before = filter.position_variance();
    let q = [1e-6; 17];
    for _ in 0..200 {
        filter.predict(0.01, [9.81, 0.0, 0.0], q);
    }
    assert!(
        filter.position_variance() > before,
        "dead-reckoning grows the position uncertainty: {} -> {}",
        before,
        filter.position_variance()
    );
}

#[test]
fn measurement_reduces_uncertainty_and_pulls_the_estimate() {
    // Large initial position-error uncertainty; a precise 3-component fix (δp_x = 5.0, δp_y = δp_z = 0).
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [100.0; 17]);
    let before_var = filter.position_variance();
    for (i, &z) in [5.0, 0.0, 0.0].iter().enumerate() {
        let mut h = [0.0f64; 17];
        h[i] = 1.0;
        filter.update_scalar(h, z, 0.01);
    }
    assert!(
        (filter.state().position_error()[0] - 5.0).abs() < 0.1,
        "the estimate is pulled to the fix: {}",
        filter.state().position_error()[0]
    );
    assert!(
        filter.position_variance() < before_var * 0.5,
        "the fix collapses the uncertainty: {before_var} -> {}",
        filter.position_variance()
    );
}

#[test]
fn covariance_trace_is_the_sum_of_the_diagonal_and_grows_under_predict() {
    // The full-covariance trace witnesses total filter uncertainty: it starts as the sum of the initial
    // diagonal and grows as predict adds process noise + propagates the covariance.
    let init = [2.0f64; 17];
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), init);
    let expected: f64 = init.iter().sum();
    assert!(
        (filter.covariance_trace() - expected).abs() < 1e-12,
        "initial trace is the diagonal sum: {} vs {expected}",
        filter.covariance_trace()
    );
    let before = filter.covariance_trace();
    let q = [1e-3; 17];
    for _ in 0..50 {
        filter.predict(0.01, [9.81, 0.0, 0.0], q);
    }
    assert!(
        filter.covariance_trace() > before,
        "predict grows the total uncertainty: {before} -> {}",
        filter.covariance_trace()
    );
    // The trace bounds the position-error variance (a subset of the diagonal).
    assert!(
        filter.covariance_trace() >= filter.position_variance(),
        "trace must dominate the position-block variance"
    );
}

#[test]
fn closed_loop_reacquires_after_a_blackout_coast() {
    // Predict-only through a "blackout" (uncertainty accumulates), then a returning GNSS position fix
    // (three scalar updates) collapses the position variance — the reacquisition the flagship needs.
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]);
    let q = [1e-6; 17];
    for _ in 0..500 {
        filter.predict(0.01, [9.81, 0.0, 0.0], q);
    }
    let var_blackout = filter.position_variance();
    assert!(
        var_blackout > 1.0,
        "uncertainty accumulated in the coast: {var_blackout}"
    );

    for i in 0..3 {
        let mut h = [0.0f64; 17];
        h[i] = 1.0;
        filter.update_scalar(h, 0.0, 0.01); // GNSS position fix (0.1 m std)
    }
    let var_reacq = filter.position_variance();
    assert!(
        var_blackout > 10.0 * var_reacq,
        "reacquisition collapses the position variance: {var_blackout} -> {var_reacq}"
    );
}
