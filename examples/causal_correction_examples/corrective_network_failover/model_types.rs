/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the network-failover corrective control loop.

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

/// One tick is one second of operations. `N_TICKS = 30` covers half a
/// minute, long enough for the primary switch to fail and for the
/// closed-loop monitor to detect the outage and reroute.
pub const N_TICKS: u32 = 30;

/// Switch identifier. The active value sits in the chain's value channel
/// between ticks. Interventions replace it with a peer.
pub type SwitchId = u64;

/// Per-tick traffic accounting and the accumulated outage record.
#[derive(Debug, Default, Clone)]
pub struct NetworkState {
    pub tick: u32,
    pub delivered_per_tick: Vec<u64>,
    pub active_switch_history: Vec<SwitchId>,
    pub packets_delivered_total: u64,
    pub packets_dropped_total: u64,
    pub failover_count: u32,
    pub primary_down_at: Option<u32>,
    pub failover_at: Option<u32>,
    pub outage_threshold_reached_at: Option<u32>,
}

/// The network plan, failure schedule and outage threshold the steps read, one `Data` contextoid
/// per fact. Every fact is a non-negative integer: two switch ids, a packet rate, a tick index and a
/// packet count, so the payload is `u64`, which holds each exactly. The context holds no position,
/// clock or event, so its spatial, temporal and spacetime slots are empty.
pub type NetworkContext = Context<Data<u64>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node index: offered traffic per tick (packets per second).
pub const TRAFFIC_PER_TICK: usize = 0;
/// Node index: id of the primary switch, which carries the traffic at the start.
pub const PRIMARY_ID: usize = 1;
/// Node index: id of the standby switch, the failover target.
pub const STANDBY_ID: usize = 2;
/// Node index: the tick at which the primary switch is scheduled to fail. From this tick onward it
/// drops every packet routed through it.
pub const PRIMARY_FAILURE_TICK: usize = 3;
/// Node index: cumulative drops (packets) above which the outage is recorded as a service breach.
/// The stage reads this on every tick.
pub const OUTAGE_DROP_THRESHOLD: usize = 4;

/// The nominal network plan, added in node-index order: node `i` holds contextoid id `i + 1`.
pub fn nominal_network_context() -> Result<NetworkContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "network plan", 5);
    for (id, value) in [
        (1, 1000), // TRAFFIC_PER_TICK: 1000 pps offered load
        (2, 0),    // PRIMARY_ID: sw0
        (3, 1),    // STANDBY_ID: sw1
        (4, 5),    // PRIMARY_FAILURE_TICK
        (5, 3000), // OUTAGE_DROP_THRESHOLD: 3 ticks of total loss
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read one `Data` contextoid's payload out of the network context, or name the node it lacks.
pub fn read(context: &NetworkContext, index: usize) -> Result<u64, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| {
            CausalityError::MissingParameter(format!("network context Datoid at node {index}"))
        })
}

pub type NetworkProcess<T> = PropagatingProcess<T, NetworkState, NetworkContext>;
