/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the lane-keeping corrective control loop.

#![allow(dead_code)] // Domain fields kept for narrative clarity even if not all are read.

use deep_causality_core::PropagatingProcess;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need wrapping
/// in `FloatType::from(…)` to switch away from `f64`.
pub type FloatType = f64;

/// Total number of simulation ticks (one tick = 0.1 s of real time).
pub const N_TICKS: u32 = 60;

/// Per-tick lateral drift schedule. Positive values push the vehicle right
/// of centre. A slow constant drift plus a small sinusoid mimics a crowned
/// road with crosswind gusts.
pub fn drift_at(tick: u32) -> FloatType {
    let t = tick as FloatType;
    0.06 + 0.04 * (t / 3.0).sin()
}

/// Accumulated trajectory and control statistics. Carried in the `State`
/// channel of the `PropagatingProcess`.
#[derive(Debug, Default, Clone)]
pub struct VehicleState {
    pub tick: u32,
    pub trajectory: Vec<FloatType>,
    pub correction_count: u32,
    pub max_offset_observed: FloatType,
    pub catastrophic_at: Option<u32>,
}

/// Read-only controller and lane parameters. Carried in the `Context` channel.
#[derive(Debug, Clone)]
pub struct LaneConfig {
    /// Half-width of the lane in metres. `|offset| > lane_half_width` is
    /// off-road; the trajectory is marked catastrophic from that tick on.
    pub lane_half_width: FloatType,
    /// Monitor threshold. When `|offset| > anomaly_threshold`, the closed
    /// loop fires a corrective intervention.
    pub anomaly_threshold: FloatType,
    /// Proportional correction gain. The corrected offset is
    /// `offset * (1.0 - p_gain)`. `p_gain = 1.0` snaps to centre; smaller
    /// values leave residual offset that the next tick can build on.
    pub p_gain: FloatType,
}

pub fn nominal_lane_config() -> LaneConfig {
    LaneConfig {
        lane_half_width: 1.5,    // standard 3 m lane
        anomaly_threshold: 0.30, // 30 cm of drift is the alarm point
        p_gain: 0.85,
    }
}

pub type LaneProcess<T> = PropagatingProcess<T, VehicleState, LaneConfig>;
