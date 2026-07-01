/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The strapdown-INS **error state** and its deterministic propagation — the core of the GNSS-denial
//! navigation engine (Gap-3 B2). This is the error-state Kalman filter's *nominal* error-dynamics `F`
//! matrix action; the covariance propagation and measurement update ride on top of it in later slices.
//!
//! 17 error states: position (3), velocity (3), attitude-error (3), accelerometer bias (3), gyro bias
//! (3), plus the two carried clock states (bias, drift). The load-bearing property — and what the
//! closed-loop navigation gate rests on — is the **inertial drift growth law through the blackout**: a
//! constant accelerometer bias grows the position error as `t²`, a constant gyro bias as `t³` (the bias
//! tilts the attitude, which mis-projects the specific force). Keeping the bias + clock states alive
//! through the coast is what gives rapid reacquisition when GNSS returns.
//!
//! Representative-frame model (Tier-A): no Earth rotation / transport rate; the body→nav DCM is taken as
//! identity over a step (`C ≈ I`), so the linearised error dynamics are
//! `δṗ = δv`, `δv̇ = −(f × δψ) − b_a`, `δψ̇ = −b_g`, biases constant (their random walk lives in the
//! process noise `Q`, not this deterministic step), `δṫ = δf`, `δḟ = 0`.
//!
//! # References
//! * Groves, P. D., *Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems*,
//!   2nd ed., Artech House (2013) — the error-state (ψ-angle) INS model and the `t²`/`t³` drift laws.

use deep_causality_num::RealField;

/// The 17-element strapdown-INS error state carried through the filter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InsErrorState<R> {
    position: [R; 3],
    velocity: [R; 3],
    attitude: [R; 3],
    accel_bias: [R; 3],
    gyro_bias: [R; 3],
    clock_bias: R,
    clock_drift: R,
}

impl<R> InsErrorState<R>
where
    R: RealField,
{
    /// The zero error state (a perfectly-known nominal).
    pub fn zero() -> Self {
        let z = [R::zero(); 3];
        Self {
            position: z,
            velocity: z,
            attitude: z,
            accel_bias: z,
            gyro_bias: z,
            clock_bias: R::zero(),
            clock_drift: R::zero(),
        }
    }

    /// An error state seeded with the given accelerometer and gyro biases (the errors that drive the
    /// inertial drift through blackout), everything else zero.
    pub fn from_biases(accel_bias: [R; 3], gyro_bias: [R; 3]) -> Self {
        Self {
            accel_bias,
            gyro_bias,
            ..Self::zero()
        }
    }

    /// Seed the carried clock error (bias, drift).
    pub fn with_clock(mut self, clock_bias: R, clock_drift: R) -> Self {
        self.clock_bias = clock_bias;
        self.clock_drift = clock_drift;
        self
    }

    /// Propagate the error state one step of `dt` under the current specific force `f` (the nav-frame
    /// accelerometer reaction, e.g. gravity + thrust), via the linearised INS error dynamics.
    pub fn propagate(&self, dt: R, specific_force: [R; 3]) -> Self {
        let f = specific_force;
        let psi = self.attitude;
        // f × δψ
        let fx = [
            f[1] * psi[2] - f[2] * psi[1],
            f[2] * psi[0] - f[0] * psi[2],
            f[0] * psi[1] - f[1] * psi[0],
        ];
        let mut out = *self;
        out.position = core::array::from_fn(|i| self.position[i] + self.velocity[i] * dt);
        out.velocity =
            core::array::from_fn(|i| self.velocity[i] + (-fx[i] - self.accel_bias[i]) * dt);
        out.attitude = core::array::from_fn(|i| self.attitude[i] + (-self.gyro_bias[i]) * dt);
        // biases are constant over the deterministic step.
        out.clock_bias = self.clock_bias + self.clock_drift * dt;
        out.clock_drift = self.clock_drift;
        out
    }

    /// Position error (3) — the navigation output the drift laws act on.
    pub fn position_error(&self) -> [R; 3] {
        self.position
    }
    /// Velocity error (3).
    pub fn velocity_error(&self) -> [R; 3] {
        self.velocity
    }
    /// Attitude error (3, small-angle).
    pub fn attitude_error(&self) -> [R; 3] {
        self.attitude
    }
    /// Accelerometer bias (3).
    pub fn accel_bias(&self) -> [R; 3] {
        self.accel_bias
    }
    /// Gyro bias (3).
    pub fn gyro_bias(&self) -> [R; 3] {
        self.gyro_bias
    }
    /// Carried clock bias.
    pub fn clock_bias(&self) -> R {
        self.clock_bias
    }
    /// Carried clock drift.
    pub fn clock_drift(&self) -> R {
        self.clock_drift
    }

    /// Pack the error state into its 17-vector form (the order the covariance filter uses):
    /// `[pos(3), vel(3), att(3), accel_bias(3), gyro_bias(3), clock_bias, clock_drift]`.
    pub fn to_array(&self) -> [R; 17] {
        [
            self.position[0],
            self.position[1],
            self.position[2],
            self.velocity[0],
            self.velocity[1],
            self.velocity[2],
            self.attitude[0],
            self.attitude[1],
            self.attitude[2],
            self.accel_bias[0],
            self.accel_bias[1],
            self.accel_bias[2],
            self.gyro_bias[0],
            self.gyro_bias[1],
            self.gyro_bias[2],
            self.clock_bias,
            self.clock_drift,
        ]
    }

    /// Rebuild the error state from its 17-vector form (inverse of [`to_array`](Self::to_array)).
    pub fn from_array(a: [R; 17]) -> Self {
        Self {
            position: [a[0], a[1], a[2]],
            velocity: [a[3], a[4], a[5]],
            attitude: [a[6], a[7], a[8]],
            accel_bias: [a[9], a[10], a[11]],
            gyro_bias: [a[12], a[13], a[14]],
            clock_bias: a[15],
            clock_drift: a[16],
        }
    }

    /// The Euclidean norm of the position error — the scalar drift the closed-loop gate reads.
    pub fn position_error_norm(&self) -> R {
        (self.position[0] * self.position[0]
            + self.position[1] * self.position[1]
            + self.position[2] * self.position[2])
            .sqrt()
    }
}
