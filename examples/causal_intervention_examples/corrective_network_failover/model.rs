/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Traffic forwarding stage and corrective driver loops.

use crate::model_types::{
    NetworkPlan, NetworkProcess, NetworkState, PRIMARY_SWITCH, SwitchId, nominal_network_plan,
};
use deep_causality_core::{EffectLog, EffectValue};
use deep_causality_haft::LogAddEntry;

/// Reports whether a given switch is up at the given tick. The primary
/// goes down at the scheduled failure tick and stays down. The standby
/// is always available.
fn switch_is_up(switch: SwitchId, tick: u32, plan: &NetworkPlan) -> bool {
    if switch == plan.primary_id {
        tick < plan.primary_failure_tick
    } else {
        true
    }
}

/// One simulation tick. The value channel carries the *currently active
/// switch id*. The stage attempts to forward the offered traffic
/// through that switch. If the switch is up, packets are delivered;
/// otherwise they are dropped. The carrier value is preserved across
/// the tick, so without an intervention the same active switch sees
/// every subsequent tick of traffic.
pub fn forward_traffic(
    value: EffectValue<SwitchId>,
    mut state: NetworkState,
    ctx: Option<NetworkPlan>,
) -> NetworkProcess<SwitchId> {
    let plan = ctx.clone().expect("NetworkPlan required");
    let active = value.into_value().unwrap_or(plan.primary_id);

    let (delivered, dropped) = if switch_is_up(active, state.tick, &plan) {
        (plan.traffic_per_tick, 0u32)
    } else {
        (0u32, plan.traffic_per_tick)
    };

    state.tick += 1;
    state.delivered_per_tick.push(delivered);
    state.active_switch_history.push(active);
    state.packets_delivered_total += u64::from(delivered);
    state.packets_dropped_total += u64::from(dropped);
    if active == plan.primary_id
        && !switch_is_up(plan.primary_id, state.tick - 1, &plan)
        && state.primary_down_at.is_none()
    {
        state.primary_down_at = Some(state.tick);
    }
    if state.outage_threshold_reached_at.is_none()
        && state.packets_dropped_total >= plan.outage_drop_threshold
    {
        state.outage_threshold_reached_at = Some(state.tick);
    }

    let mut logs = EffectLog::new();
    let marker = if dropped > 0 { " [DROPS]" } else { "" };
    logs.add_entry(&format!(
        "tick {:>2}: active = sw{}, delivered = {:>4}, dropped = {:>4}{}",
        state.tick, active, delivered, dropped, marker
    ));

    NetworkProcess::<SwitchId> {
        value: EffectValue::Value(active),
        state,
        context: ctx,
        error: None,
        logs,
    }
}

pub fn initial_process() -> NetworkProcess<SwitchId> {
    NetworkProcess::<SwitchId> {
        value: EffectValue::Value(PRIMARY_SWITCH),
        state: NetworkState::default(),
        context: Some(nominal_network_plan()),
        error: None,
        logs: EffectLog::new(),
    }
}
