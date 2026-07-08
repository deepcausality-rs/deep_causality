/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The 17-state **error-state Kalman filter** (ESKF) covariance propagation and measurement update —
//! the estimator layer over the [`InsErrorState`](super::InsErrorState) error-dynamics (Gap-3 B2).
//!
//! Predict propagates the error-state estimate (`InsErrorState::propagate`) and its covariance
//! `P ← F·P·Fᵀ + Q`, where `F` is the linearised error-state transition (the same linearisation as the
//! state propagation — gated by `f_matrix_matches_propagate`). Measurements are folded in as **sequential
//! scalar updates** (the standard decorrelated-measurement trick), so the innovation covariance `S` is a
//! scalar and no matrix inversion is needed — a position fix is three scalar updates, a pseudorange one.
//! The load-bearing behaviour: uncertainty grows while dead-reckoning through blackout (predict), and a
//! returning GNSS/optical measurement collapses it and pulls the error estimate back (reacquisition).
//!
//! # References
//! * Groves, P. D., *Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems*,
//!   2nd ed., Artech House (2013), §14 (error-state / sequential Kalman update).

use super::ins_error_state::InsErrorState;
use deep_causality_algebra::RealField;

/// The error-state dimension (17 = INS 15-state + clock bias/drift).
pub const NAV_STATES: usize = 17;

// ── small fixed-size dense linear algebra (generic over the square size, clippy-clean) ────────────

fn mat_mul<R: RealField, const M: usize>(a: &[[R; M]; M], b: &[[R; M]; M]) -> [[R; M]; M] {
    core::array::from_fn(|i| {
        core::array::from_fn(|j| (0..M).fold(R::zero(), |s, k| s + a[i][k] * b[k][j]))
    })
}
fn mat_transpose<R: RealField, const M: usize>(a: &[[R; M]; M]) -> [[R; M]; M] {
    core::array::from_fn(|i| core::array::from_fn(|j| a[j][i]))
}
fn mat_add<R: RealField, const M: usize>(a: &[[R; M]; M], b: &[[R; M]; M]) -> [[R; M]; M] {
    core::array::from_fn(|i| core::array::from_fn(|j| a[i][j] + b[i][j]))
}
fn mat_vec<R: RealField, const M: usize>(a: &[[R; M]; M], v: &[R; M]) -> [R; M] {
    core::array::from_fn(|i| (0..M).fold(R::zero(), |s, k| s + a[i][k] * v[k]))
}
fn dot<R: RealField, const M: usize>(a: &[R; M], b: &[R; M]) -> R {
    (0..M).fold(R::zero(), |s, k| s + a[k] * b[k])
}
fn diag<R: RealField, const M: usize>(d: &[R; M]) -> [[R; M]; M] {
    core::array::from_fn(|i| core::array::from_fn(|j| if i == j { d[i] } else { R::zero() }))
}

/// The linearised 17-state error-state transition matrix `F` for one step of `dt` under specific force
/// `f`. Reproduces [`InsErrorState::propagate`] exactly (`F·x == propagate(x)`), so the covariance and the
/// state advance under one consistent linearisation.
pub fn nav_transition_matrix<R: RealField>(dt: R, f: [R; 3]) -> [[R; NAV_STATES]; NAV_STATES] {
    let mut m: [[R; NAV_STATES]; NAV_STATES] = core::array::from_fn(|i| {
        core::array::from_fn(|j| if i == j { R::one() } else { R::zero() })
    });
    let neg = R::zero() - R::one();
    // δp ← δv
    m[0][3] = dt;
    m[1][4] = dt;
    m[2][5] = dt;
    // δv ← δψ : −[f×]·dt, with −[f×] = [[0, fz, −fy], [−fz, 0, fx], [fy, −fx, 0]]
    m[3][7] = f[2] * dt;
    m[3][8] = neg * f[1] * dt;
    m[4][6] = neg * f[2] * dt;
    m[4][8] = f[0] * dt;
    m[5][6] = f[1] * dt;
    m[5][7] = neg * f[0] * dt;
    // δv ← accel bias : −I·dt
    m[3][9] = neg * dt;
    m[4][10] = neg * dt;
    m[5][11] = neg * dt;
    // δψ ← gyro bias : −I·dt
    m[6][12] = neg * dt;
    m[7][13] = neg * dt;
    m[8][14] = neg * dt;
    // clock bias ← clock drift
    m[15][16] = dt;
    m
}

/// A 17-state error-state Kalman filter: the error-state estimate + its covariance.
#[derive(Clone, Debug)]
pub struct NavFilter<R: RealField> {
    state: InsErrorState<R>,
    cov: [[R; NAV_STATES]; NAV_STATES],
}

impl<R: RealField> NavFilter<R> {
    /// Build the filter from an initial error-state estimate and an initial covariance diagonal.
    pub fn new(state: InsErrorState<R>, cov_diag: [R; NAV_STATES]) -> Self {
        Self {
            state,
            cov: diag(&cov_diag),
        }
    }

    /// Predict one step: propagate the error state and `P ← F·P·Fᵀ + Q` (`Q` = the process-noise
    /// diagonal — IMU random walk + clock noise, inflated during buffet).
    pub fn predict(&mut self, dt: R, specific_force: [R; 3], process_noise_diag: [R; NAV_STATES]) {
        self.state = self.state.propagate(dt, specific_force);
        let f = nav_transition_matrix(dt, specific_force);
        let fp = mat_mul(&f, &self.cov);
        let fpft = mat_mul(&fp, &mat_transpose(&f));
        self.cov = mat_add(&fpft, &diag(&process_noise_diag));
    }

    /// Fold in one scalar measurement `z = h·δx + noise` with measurement variance `r` (a sequential
    /// scalar update; `S = h·P·hᵀ + r` is a scalar, so no inversion). Corrects the estimate and shrinks
    /// the covariance.
    ///
    /// The covariance update is the **Joseph form** `P ← (I−K·h)·P·(I−K·h)ᵀ + r·K⊗K`, followed by a
    /// re-symmetrization. The simple form `P − K⊗(h·P)` loses symmetry and positive-definiteness
    /// under long sequences of near-unity-gain folds (a precise receiver folded every step), after
    /// which the cross-term gains change sign and the injected corrections diverge; Joseph is
    /// PSD-preserving unconditionally (Groves 2013, §3.4.3).
    pub fn update_scalar(&mut self, h: [R; NAV_STATES], z: R, r: R) {
        let x = self.state.to_array();
        let ph = mat_vec(&self.cov, &h); // P·hᵀ
        let s = dot(&h, &ph) + r; // innovation covariance (scalar)
        let innov = z - dot(&h, &x);
        let k: [R; NAV_STATES] = core::array::from_fn(|i| ph[i] / s); // Kalman gain
        let x_new: [R; NAV_STATES] = core::array::from_fn(|i| x[i] + k[i] * innov);
        self.state = InsErrorState::from_array(x_new);
        // Joseph form: A = I − K⊗h; P ← A·P·Aᵀ + r·K⊗K, then symmetrize.
        let a: [[R; NAV_STATES]; NAV_STATES] = core::array::from_fn(|i| {
            core::array::from_fn(|j| {
                let id = if i == j { R::one() } else { R::zero() };
                id - k[i] * h[j]
            })
        });
        let ap = mat_mul(&a, &self.cov);
        let apat = mat_mul(&ap, &mat_transpose(&a));
        let half = R::one() / (R::one() + R::one());
        self.cov = core::array::from_fn(|i| {
            core::array::from_fn(|j| {
                let joseph = apat[i][j] + r * k[i] * k[j];
                let joseph_t = apat[j][i] + r * k[j] * k[i];
                (joseph + joseph_t) * half
            })
        });
    }

    /// The full error-state covariance matrix (snapshot access; diagnostics use
    /// [`position_variance`](Self::position_variance) / [`covariance_trace`](Self::covariance_trace)).
    pub fn covariance(&self) -> &[[R; NAV_STATES]; NAV_STATES] {
        &self.cov
    }

    /// Rebuild a filter from snapshotted state and covariance: the exact inverse of reading
    /// [`state`](Self::state) and [`covariance`](Self::covariance). Exists for the
    /// state-snapshot resume path.
    pub fn restore(state: InsErrorState<R>, cov: [[R; NAV_STATES]; NAV_STATES]) -> Self {
        Self { state, cov }
    }

    /// The current error-state estimate.
    pub fn state(&self) -> &InsErrorState<R> {
        &self.state
    }

    /// Apply the ESKF feedback reset: zero the navigation-error part of the estimate (position,
    /// velocity, attitude) after it has been injected into the nominal trajectory. The learned bias and
    /// clock states persist. (The covariance is unchanged — the reset moves the *mean*, not the spread.)
    pub fn reset_navigation_error(&mut self) {
        self.state = self.state.reset_navigation();
    }

    /// The position-error variance (trace of the 3×3 position block) — the reacquisition witness.
    pub fn position_variance(&self) -> R {
        self.cov[0][0] + self.cov[1][1] + self.cov[2][2]
    }

    /// The full covariance trace (total filter uncertainty).
    pub fn covariance_trace(&self) -> R {
        (0..NAV_STATES).fold(R::zero(), |s, i| s + self.cov[i][i])
    }
}
