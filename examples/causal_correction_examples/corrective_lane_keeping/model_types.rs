/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the lane-keeping corrective control loop.

#![allow(dead_code)] // Domain fields kept for narrative clarity even if not all are read.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalityError, PropagatingProcess};
use deep_causality_num::Lift;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need lifting
/// through `deep_causality_num::lift` to switch away from `f64`.
pub type FloatType = f64;

/// Total number of simulation ticks (one tick = 0.1 s of real time).
pub const N_TICKS: u32 = 60;

/// Per-tick lateral drift schedule. Positive values push the vehicle right
/// of centre. A slow constant drift plus a small sinusoid mimics a crowned
/// road with crosswind gusts; the base drift, the gust amplitude and the gust
/// period are read from `lane`.
pub fn drift_at(tick: u32, lane: &LaneContext) -> Result<FloatType, CausalityError> {
    let t = tick.lift::<FloatType>();
    Ok(
        read(lane, BASE_DRIFT)?
            + read(lane, GUST_AMPLITUDE)? * (t / read(lane, GUST_PERIOD)?).sin(),
    )
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

/// The lane, controller and drift facts the steps read, one `Data` contextoid per quantity. The context
/// holds no position, clock or event, so its spatial, temporal and spacetime slots are empty.
pub type LaneContext = Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: half-width of the lane in metres. `|offset| > lane_half_width` is off-road; the
/// trajectory is marked catastrophic from that tick on.
pub const LANE_HALF_WIDTH: ContextoidId = 1;
/// Contextoid id: monitor threshold in metres. When `|offset| > anomaly_threshold`, the closed
/// loop fires a corrective intervention.
pub const ANOMALY_THRESHOLD: ContextoidId = 2;
/// Contextoid id: proportional correction gain. The corrected offset is
/// `offset * (1.0 - p_gain)`. `p_gain = 1.0` snaps to centre; smaller values leave residual offset
/// that the next tick can build on.
pub const P_GAIN: ContextoidId = 3;
/// Contextoid id: constant lateral drift from the road crown (m per tick).
pub const BASE_DRIFT: ContextoidId = 4;
/// Contextoid id: amplitude of the crosswind gusts (m per tick).
pub const GUST_AMPLITUDE: ContextoidId = 5;
/// Contextoid id: period of the crosswind gusts (ticks per radian).
pub const GUST_PERIOD: ContextoidId = 6;

/// The nominal lane, controller and drift: one `Data` contextoid per fact, keyed by its
/// contextoid id.
pub fn nominal_lane_context() -> Result<LaneContext, ContextIndexError> {
    let facts = [
        (LANE_HALF_WIDTH, 1.5),    // standard 3 m lane
        (ANOMALY_THRESHOLD, 0.30), // 30 cm of drift is the alarm point
        (P_GAIN, 0.85),
        (BASE_DRIFT, 0.06),
        (GUST_AMPLITUDE, 0.04),
        (GUST_PERIOD, 3.0),
    ];
    let mut context = Context::with_capacity(1, "lane", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read the payload of the `Data` contextoid `id` out of the lane context, or name the id it
/// lacks.
pub fn read(context: &LaneContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError::MissingParameter(format!("lane context Datoid with contextoid id {id}"))
    })
}

pub type LaneProcess<T> = PropagatingProcess<T, VehicleState, LaneContext>;
