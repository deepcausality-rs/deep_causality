/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The 17-state ESKF (`NavFilter`): the linearised transition matrix is consistent with the state
//! propagation, predict grows the position uncertainty (dead-reckoning through blackout), a measurement
//! shrinks it and pulls the estimate toward the fix, and the closed loop reacquires — the load-bearing
//! filter behaviour the navigation gate rests on.

use deep_causality_cfd::{InsErrorState, NAV_STATES, NavFilter, nav_transition_matrix};

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
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]).unwrap();
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
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [100.0; 17]).unwrap();
    let before_var = filter.position_variance();
    for (i, &z) in [5.0, 0.0, 0.0].iter().enumerate() {
        let mut h = [0.0f64; 17];
        h[i] = 1.0;
        filter.update_scalar(h, z, 0.01).unwrap();
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
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), init).unwrap();
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
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]).unwrap();
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
        filter.update_scalar(h, 0.0, 0.01).unwrap(); // GNSS position fix (0.1 m std)
    }
    let var_reacq = filter.position_variance();
    assert!(
        var_blackout > 10.0 * var_reacq,
        "reacquisition collapses the position variance: {var_blackout} -> {var_reacq}"
    );
}

// ── Covariance validation at the entry points (design D3) ─────────────────────────────────────────

#[test]
fn new_rejects_a_negative_variance() {
    // A diagonal is a vector of variances; a negative one is not a covariance.
    let mut diag = [1.0f64; NAV_STATES];
    diag[5] = -1.0;
    assert!(
        NavFilter::new(InsErrorState::<f64>::zero(), diag).is_err(),
        "a negative variance must be refused at construction"
    );
}

#[test]
fn new_rejects_a_non_finite_variance() {
    let mut diag = [1.0f64; NAV_STATES];
    diag[0] = f64::NAN;
    assert!(NavFilter::new(InsErrorState::<f64>::zero(), diag).is_err());
    let mut diag = [1.0f64; NAV_STATES];
    diag[2] = f64::INFINITY;
    assert!(NavFilter::new(InsErrorState::<f64>::zero(), diag).is_err());
}

#[test]
fn new_accepts_a_zero_variance_on_the_boundary() {
    // Zero is a non-negative variance: PSD includes the boundary, so `new` admits it. The degenerate
    // *update* (s = 0) is what the guarded `update_scalar` refuses, not construction.
    let diag = [0.0f64; NAV_STATES];
    assert!(NavFilter::new(InsErrorState::<f64>::zero(), diag).is_ok());
}

#[test]
fn restore_rejects_an_asymmetric_covariance() {
    let mut cov = [[0.0f64; NAV_STATES]; NAV_STATES];
    for (i, row) in cov.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    // A single-sided cross-covariance: [0][3] set, [3][0] left zero — asymmetric beyond any tolerance.
    cov[0][3] = 10.0;
    assert!(
        NavFilter::restore(InsErrorState::<f64>::zero(), cov).is_err(),
        "an asymmetric matrix is not a covariance"
    );
}

#[test]
fn restore_rejects_a_negative_diagonal() {
    let mut cov = [[0.0f64; NAV_STATES]; NAV_STATES];
    for (i, row) in cov.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    cov[7][7] = -1.0;
    assert!(NavFilter::restore(InsErrorState::<f64>::zero(), cov).is_err());
}

#[test]
fn restore_rejects_a_non_finite_entry() {
    let mut cov = [[0.0f64; NAV_STATES]; NAV_STATES];
    for (i, row) in cov.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    cov[9][9] = f64::NAN;
    assert!(NavFilter::restore(InsErrorState::<f64>::zero(), cov).is_err());
}

#[test]
fn restore_admits_float_level_asymmetry_within_tolerance() {
    // A cross-covariance that is symmetric only to a float ULP must still restore: exact equality is
    // the wrong test for a matrix that has round-tripped through serialisation.
    let mut cov = [[0.0f64; NAV_STATES]; NAV_STATES];
    for (i, row) in cov.iter_mut().enumerate() {
        row[i] = 2500.0;
    }
    cov[0][3] = 300.0;
    cov[3][0] = 300.0 + 300.0 * f64::EPSILON; // one ULP of asymmetry at this scale
    assert!(
        NavFilter::restore(InsErrorState::<f64>::zero(), cov).is_ok(),
        "float-level asymmetry within √ε must be admitted"
    );
}

#[test]
fn a_valid_filter_snapshots_and_restores_exactly() {
    // Build a filter, evolve it (predict + a fold), snapshot its state and covariance, and restore:
    // the restored filter must be bit-identical to the suspended one.
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [2500.0; NAV_STATES]).unwrap();
    let q = [1e-4; NAV_STATES];
    for _ in 0..25 {
        filter.predict(0.1, [9.81, 0.1, -0.2], q);
    }
    for i in 0..3 {
        let mut h = [0.0f64; NAV_STATES];
        h[i] = 1.0;
        filter.update_scalar(h, 3.0, 1.0).unwrap();
    }
    let state = *filter.state();
    let cov = *filter.covariance();
    let restored = NavFilter::restore(state, cov).expect("a valid snapshot must restore");
    assert_eq!(
        restored.state(),
        filter.state(),
        "state must round-trip exactly"
    );
    assert_eq!(
        restored.covariance(),
        filter.covariance(),
        "covariance must round-trip exactly"
    );
}

// ── The guarded measurement update (design D2) ────────────────────────────────────────────────────

#[test]
fn a_degenerate_update_is_refused_and_leaves_the_filter_untouched() {
    // Reachable from the public API: a zero-variance position axis met with a zero-variance fix gives
    // s = P[0][0] + r = 0. The update must refuse and leave state + covariance exactly as they were.
    let mut diag = [1.0f64; NAV_STATES];
    diag[0] = 0.0; // zero variance on position axis x (admitted at the PSD boundary)
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), diag).unwrap();
    let state_before = *filter.state();
    let cov_before = *filter.covariance();
    let mut h = [0.0f64; NAV_STATES];
    h[0] = 1.0;
    let result = filter.update_scalar(h, 3.0, 0.0); // r = 0 with P[0][0] = 0 → s = 0 → NaN gain
    assert!(
        result.is_err(),
        "a degenerate update (s = 0) must be refused"
    );
    assert_eq!(
        filter.state(),
        &state_before,
        "state must be untouched after refusal"
    );
    assert_eq!(
        filter.covariance(),
        &cov_before,
        "covariance must be untouched after refusal"
    );
}

#[test]
fn update_scalar_refuses_a_negative_variance() {
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; NAV_STATES]).unwrap();
    let state_before = *filter.state();
    let cov_before = *filter.covariance();
    let mut h = [0.0f64; NAV_STATES];
    h[0] = 1.0;
    assert!(
        filter.update_scalar(h, 1.0, -1.0).is_err(),
        "a negative measurement variance must be refused"
    );
    assert_eq!(filter.state(), &state_before);
    assert_eq!(filter.covariance(), &cov_before);
}

#[test]
fn a_refused_update_leaves_the_run_able_to_continue() {
    // After a refusal, a subsequent well-posed fix still folds — the filter is not poisoned.
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [4.0; NAV_STATES]).unwrap();
    let mut h = [0.0f64; NAV_STATES];
    h[0] = 1.0;
    assert!(filter.update_scalar(h, 5.0, -1.0).is_err()); // refused (negative r)
    filter.update_scalar(h, 5.0, 1.0).unwrap(); // valid fold still works
    assert!(
        (filter.state().position_error()[0] - 4.0).abs() < 1e-9,
        "a valid fold after a refusal still pulls the estimate"
    );
}

#[test]
fn a_valid_update_is_bit_identical_after_the_guard() {
    // The guard is a pure early-return before the (unchanged) Joseph arithmetic, so a well-posed fold
    // is byte-for-byte what it was before this change. Pinned as an exact-f64 regression: h = e_0,
    // P = diag(4.0), z = 5.0, r = 1.0 → k_0 = 4/5, x_new_0 = 4.0, P[0][0] = (1−k_0)²·4 + k_0².
    let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [4.0; NAV_STATES]).unwrap();
    let mut h = [0.0f64; NAV_STATES];
    h[0] = 1.0;
    filter.update_scalar(h, 5.0, 1.0).unwrap();
    assert_eq!(
        filter.state().position_error()[0],
        4.0,
        "estimate pulled to the fix (exact)"
    );
    // cov[0][0] = (1−4/5)²·4 + 1·(4/5)²; cov[1][1] = cov[2][2] = 4.0 (untouched diagonal).
    assert_eq!(filter.covariance()[0][0], EXPECTED_COV00);
    assert_eq!(filter.position_variance(), EXPECTED_COV00 + 8.0);
}

// ── The process-noise discretisation (design D1): Q_d = Q_c·dt ────────────────────────────────────

#[test]
fn covariance_growth_is_invariant_under_step_refinement() {
    // Over a fixed horizon T, with Q_d = Q_c·dt, the accumulated process noise is a function of elapsed
    // time, not step count. The accelerometer-bias axis (index 9) is a *leaf* of the transition matrix
    // (F row 9 = e_9, no incoming coupling), so its variance accumulates exactly T·Q_c; halving dt must
    // leave it unchanged. Under the old per-step Q it would double.
    let horizon = 1.0f64;
    let qc = 3.0e-3; // spectral density on the accel-bias axis
    let mut q = [0.0f64; NAV_STATES];
    q[9] = qc;
    let run = |dt: f64| {
        let n = (horizon / dt).round() as usize;
        let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [0.0; NAV_STATES]).unwrap();
        for _ in 0..n {
            filter.predict(dt, [0.0, 0.0, 0.0], q);
        }
        filter.covariance()[9][9]
    };
    let coarse = run(0.01); // 100 steps
    let fine = run(0.005); // 200 steps
    let expected = horizon * qc; // T·Q_c
    assert!(
        (coarse - expected).abs() < 1e-12,
        "leaf variance is T·Q_c (elapsed time, not step count): {coarse} vs {expected}"
    );
    assert!(
        (coarse - fine).abs() < 1e-12,
        "terminal covariance is invariant under halving dt: {coarse} vs {fine}"
    );
}

#[test]
fn changing_dt_alone_does_not_retune_the_filter() {
    // The position witness the examples gate on. With only a position spectral density (no velocity
    // noise, no seeded cross-covariance), the position block is also a leaf, so its terminal variance
    // over a fixed horizon is T·Q_c regardless of the step size — the filter is not silently re-tuned by
    // a change of dt. (Under the pre-fix per-step Q, halving dt doubled this.)
    let horizon = 2.0f64;
    let qc = 5.0e-4;
    let mut q = [0.0f64; NAV_STATES];
    q[0] = qc; // position-x spectral density only
    let run = |dt: f64| {
        let n = (horizon / dt).round() as usize;
        let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [0.0; NAV_STATES]).unwrap();
        for _ in 0..n {
            filter.predict(dt, [0.0, 0.0, 0.0], q);
        }
        filter.covariance()[0][0]
    };
    let coarse = run(0.02);
    let fine = run(0.01);
    assert!(
        (coarse - fine).abs() < 1e-12,
        "position variance over a fixed horizon is dt-invariant: {coarse} vs {fine}"
    );
    assert!(
        (coarse - horizon * qc).abs() < 1e-12,
        "and equals T·Q_c: {coarse} vs {}",
        horizon * qc
    );
}

/// Exact `f64` value of `cov[0][0]` after the fold, reproduced in the **same operation order** the
/// Joseph update uses (`((1−k0)·4)·(1−k0) + (1·k0)·k0`), so it is bit-identical to the implementation by
/// construction. Pins the valid-path arithmetic: any reordering or algebraic change is caught, and the
/// D2 guard — a pure early-return before this arithmetic — leaves it byte-unchanged.
const EXPECTED_COV00: f64 = {
    let k0 = 4.0 / 5.0;
    ((1.0 - k0) * 4.0) * (1.0 - k0) + (1.0 * k0) * k0
};
