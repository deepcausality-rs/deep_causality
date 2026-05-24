/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the network-failover corrective control loop.

#![allow(dead_code)] // Domain fields kept for narrative clarity even if not all are read.

use deep_causality_core::PropagatingProcess;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need wrapping
/// in `FloatType::from(…)` to switch away from `f64`.
pub type FloatType = f64;

/// One tick is one second of operations. `N_TICKS = 30` covers half a
/// minute, long enough for the primary switch to fail and for the
/// closed-loop monitor to detect the outage and reroute.
pub const N_TICKS: u32 = 30;

/// Switch identifier. The active value sits in the chain's value channel
/// between ticks. Interventions replace it with a peer.
pub type SwitchId = u32;

pub const PRIMARY_SWITCH: SwitchId = 0;
pub const STANDBY_SWITCH: SwitchId = 1;

/// Per-tick traffic accounting and the accumulated outage record.
#[derive(Debug, Default, Clone)]
pub struct NetworkState {
    pub tick: u32,
    pub delivered_per_tick: Vec<u32>,
    pub active_switch_history: Vec<SwitchId>,
    pub packets_delivered_total: u64,
    pub packets_dropped_total: u64,
    pub failover_count: u32,
    pub primary_down_at: Option<u32>,
    pub failover_at: Option<u32>,
    pub outage_threshold_reached_at: Option<u32>,
}

/// Read-only network plan, failure schedule, and outage thresholds.
#[derive(Debug, Clone)]
pub struct NetworkPlan {
    /// Offered traffic per tick (packets per second).
    pub traffic_per_tick: u32,
    pub primary_id: SwitchId,
    pub standby_id: SwitchId,
    /// The tick at which the primary switch is scheduled to fail. From
    /// this tick onward it drops every packet routed through it.
    pub primary_failure_tick: u32,
    /// Cumulative drops above which the outage is recorded as a service
    /// breach. The monitor reads this on every tick.
    pub outage_drop_threshold: u64,
}

pub fn nominal_network_plan() -> NetworkPlan {
    NetworkPlan {
        traffic_per_tick: 1000, // 1000 pps offered load
        primary_id: PRIMARY_SWITCH,
        standby_id: STANDBY_SWITCH,
        primary_failure_tick: 5,
        outage_drop_threshold: 3_000, // 3 ticks of total loss
    }
}

pub type NetworkProcess<T> = PropagatingProcess<T, NetworkState, NetworkPlan>;
