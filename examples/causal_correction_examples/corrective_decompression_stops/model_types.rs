/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the corrective-decompression-stops example.

#![allow(dead_code)] // Domain fields kept for narrative clarity even if not all are read.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalityError, PropagatingProcess};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need lifting
/// through `deep_causality_num::lift` to switch away from `f64`.
pub type FloatType = f64;

/// One tick is 0.5 minutes (30 seconds) of dive time.
/// `N_TICKS = 30` covers 15 minutes, long enough for the open-loop run to
/// surface in five minutes and for the closed loop to insert several
/// decompression stops before finishing.
pub const N_TICKS: u32 = 30;

/// Tick duration in minutes: the time step of the tissue kinetics.
pub const TICK_MINUTES: FloatType = 0.5;

/// Depth of the bottom phase (m). The diver starts here with the tissue
/// fully saturated to it; both are the initial values of `DiveState`.
pub const STARTING_DEPTH_M: FloatType = 30.0;

/// Inspired N2 partial pressure (bar) at depth `d` (m). Atmospheric N2
/// fraction is 79%; absolute pressure climbs by 1 bar per 10 m of depth.
pub fn inspired_n2_pp(depth: FloatType) -> FloatType {
    let ambient = 1.0 + depth / 10.0;
    0.79 * ambient
}

pub fn ambient_pressure(depth: FloatType) -> FloatType {
    1.0 + depth / 10.0
}

/// Single-compartment dive state. The `diving_decompression` example in
/// `medicine_examples` tracks 16 compartments per Bühlmann ZH-L16C; this
/// example uses one mid-range compartment because the intervention pattern
/// is the same regardless of how many compartments you carry.
#[derive(Debug, Default, Clone)]
pub struct DiveState {
    pub tick: u32,
    pub depth_m: FloatType,
    pub tissue_n2_bar: FloatType,
    pub depth_trajectory: Vec<FloatType>,
    pub tissue_trajectory: Vec<FloatType>,
    pub ratio_trajectory: Vec<FloatType>,
    pub last_ratio: FloatType,
    pub stop_count: u32,
    pub max_ratio_observed: FloatType,
    pub dcs_at: Option<u32>,
}

/// The dive plan and decompression thresholds the steps read, one `Data` contextoid per quantity.
/// The context holds no position, clock or event, so its spatial, temporal and spacetime slots are
/// empty.
pub type DiveContext = Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node index: ascent rate under a continuous-ascent plan (m/min).
pub const ASCENT_RATE: usize = 0;
/// Node index: tissue half-time (min). The compartment here is a mid-range 10-minute half-time,
/// comparable to Bühlmann compartment 2.
pub const HALF_TIME: usize = 1;
/// Node index: supersaturation ratio at which DCS risk is assumed certain.
pub const DCS_RATIO_THRESHOLD: usize = 2;
/// Node index: monitor threshold. The closed loop fires a corrective stop the moment the post-tick
/// ratio crosses this.
pub const SAFETY_RATIO_THRESHOLD: usize = 3;

/// The nominal dive plan, added in node-index order: node `i` holds contextoid id `i + 1`.
pub fn nominal_dive_context() -> Result<DiveContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "dive", 4);
    for (id, value) in [
        (1, 6.0),  // ASCENT_RATE: 3 m per 0.5-minute tick
        (2, 10.0), // HALF_TIME
        (3, 1.6),  // DCS_RATIO_THRESHOLD
        // SAFETY_RATIO_THRESHOLD: a single ascent tick can swing the ratio by roughly +0.25 at
        // this physics. The safety threshold is set well below the DCS line so a stop fires
        // before the next ascent could overshoot.
        (4, 1.15),
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read one `Data` contextoid's payload out of the dive context, or name the node it lacks.
pub fn read(context: &DiveContext, index: usize) -> Result<FloatType, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| {
            CausalityError::MissingParameter(format!("dive context Datoid at node {index}"))
        })
}

pub type DiveProcess<T> = PropagatingProcess<T, DiveState, DiveContext>;
