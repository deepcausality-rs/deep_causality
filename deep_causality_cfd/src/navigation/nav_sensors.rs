/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Synthetic sensor models for the GNSS-denial navigation loop (decision ⑥). The sensors sample the
//! simulation's own ground truth; their accuracy enters the filter as the measurement/process noise, not
//! as added randomness — so the closed-loop gates are deterministic.
//!
//! * **Strapdown IMU** — the *primary* sensor and the **drift source**: the accelerometer reports the
//!   true specific force plus a bias, so dead-reckoning through blackout accumulates the bias as `t²`
//!   position error. Its spec sets the ESKF process-noise `Q` (gyro/accel random walk).
//! * **GNSS** — a metre-level position fix, available *outside* blackout only (physics-gated off by the
//!   ④ blackout flag).
//! * **Through-plasma optical** — a coarser (~50 m) position/bearing fix that *works during blackout*,
//!   bounding the INS drift (the 2025-26 SOTA aid).
//!
//! The GNSS/optical fixes here are the ground-truth position; their 1σ accuracy is carried as the
//! measurement variance `r` passed to [`ReentryNavEngine::correct_position`](super::ReentryNavEngine).

use deep_causality_algebra::RealField;

/// A strapdown-IMU model: a constant accelerometer + gyro bias and the process noise its grade implies.
#[derive(Clone, Copy, Debug)]
pub struct ImuModel<R> {
    accel_bias: [R; 3],
    gyro_bias: [R; 3],
    process_noise_diag: [R; 17],
}

impl<R: RealField> ImuModel<R> {
    /// An IMU with the given accelerometer/gyro bias and ESKF process-noise diagonal `Q`.
    pub fn new(accel_bias: [R; 3], gyro_bias: [R; 3], process_noise_diag: [R; 17]) -> Self {
        Self {
            accel_bias,
            gyro_bias,
            process_noise_diag,
        }
    }

    /// The measured specific force: the true (aero) specific force plus the accelerometer bias — the
    /// error that makes the dead-reckoned nominal drift through blackout.
    pub fn sense_specific_force(&self, true_specific_force: [R; 3]) -> [R; 3] {
        core::array::from_fn(|i| true_specific_force[i] + self.accel_bias[i])
    }

    /// The measured body angular rate: the true rate plus the gyro bias — the `ω̂` the nominal attitude
    /// integrates. For the point-mass corridor/weather (no true rotation, `IMU_GYRO_BIAS = 0`) this is
    /// zero, so the nominal attitude stays at identity; a non-zero gyro bias would tilt the nominal, the
    /// error the ESKF's gyro-bias and attitude states exist to track.
    pub fn sense_angular_rate(&self, true_angular_rate: [R; 3]) -> [R; 3] {
        core::array::from_fn(|i| true_angular_rate[i] + self.gyro_bias[i])
    }

    /// The accelerometer bias (the drift driver).
    pub fn accel_bias(&self) -> [R; 3] {
        self.accel_bias
    }
    /// The gyro bias.
    pub fn gyro_bias(&self) -> [R; 3] {
        self.gyro_bias
    }
    /// The ESKF process-noise diagonal `Q` implied by this IMU's grade.
    pub fn process_noise(&self) -> [R; 17] {
        self.process_noise_diag
    }
}
