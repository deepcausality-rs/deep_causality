/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Traffic forwarding stage and corrective driver loops.

use crate::model_types::{
    NetworkContext, NetworkProcess, NetworkState, OUTAGE_DROP_THRESHOLD, PRIMARY_FAILURE_TICK,
    PRIMARY_ID, STANDBY_ID, SwitchId, TRAFFIC_PER_TICK, read,
};
use deep_causality_core::{AlternatableValue, CausalEffect, CausalityError, EffectLog};
use deep_causality_haft::LogAddEntry;

/// Reports whether a given switch is up at the given tick. The primary
/// goes down at the scheduled failure tick and stays down. The standby
/// is always available.
fn switch_is_up(
    switch: SwitchId,
    tick: u32,
    plan: &NetworkContext,
) -> Result<bool, CausalityError> {
    if switch == read(plan, PRIMARY_ID)? {
        Ok(u64::from(tick) < read(plan, PRIMARY_FAILURE_TICK)?)
    } else {
        Ok(true)
    }
}

/// One simulation tick. The value channel carries the *currently active
/// switch id*. The stage attempts to forward the offered traffic
/// through that switch. If the switch is up, packets are delivered;
/// otherwise they are dropped. The carrier value is preserved across
/// the tick, so without an intervention the same active switch sees
/// every subsequent tick of traffic. The plan, the failure schedule and the
/// outage threshold are read from the network context the process carries.
/// A missing switch id, context or plan fact ends the process in error with
/// the state unchanged.
pub fn forward_traffic(
    value: CausalEffect<SwitchId>,
    mut state: NetworkState,
    ctx: Option<NetworkContext>,
) -> NetworkProcess<SwitchId> {
    match advance(value, &mut state, ctx.as_ref()) {
        Ok((active, logs)) => {
            NetworkProcess::<SwitchId>::new(Ok(CausalEffect::value(active)), state, ctx, logs)
        }
        Err(err) => NetworkProcess::<SwitchId>::new(Err(err), state, ctx, EffectLog::new()),
    }
}

/// The tick itself: reads every input first, then advances `state` and
/// returns the active switch with its log entry.
fn advance(
    value: CausalEffect<SwitchId>,
    state: &mut NetworkState,
    ctx: Option<&NetworkContext>,
) -> Result<(SwitchId, EffectLog), CausalityError> {
    let active = value
        .into_value()
        .ok_or_else(CausalityError::ValueNotAvailable)?;
    let plan = ctx.ok_or_else(CausalityError::MissingContext)?;
    let primary = read(plan, PRIMARY_ID)?;
    let traffic_per_tick = read(plan, TRAFFIC_PER_TICK)?;
    let outage_drop_threshold = read(plan, OUTAGE_DROP_THRESHOLD)?;
    let active_up = switch_is_up(active, state.tick, plan)?;
    let primary_up = switch_is_up(primary, state.tick, plan)?;

    let (delivered, dropped) = if active_up {
        (traffic_per_tick, 0)
    } else {
        (0, traffic_per_tick)
    };

    state.tick += 1;
    state.delivered_per_tick.push(delivered);
    state.active_switch_history.push(active);
    state.packets_delivered_total += delivered;
    state.packets_dropped_total += dropped;
    if active == primary && !primary_up && state.primary_down_at.is_none() {
        state.primary_down_at = Some(state.tick);
    }
    if state.outage_threshold_reached_at.is_none()
        && state.packets_dropped_total >= outage_drop_threshold
    {
        state.outage_threshold_reached_at = Some(state.tick);
    }

    let mut logs = EffectLog::new();
    let marker = if dropped > 0 { " [DROPS]" } else { "" };
    logs.add_entry(&format!(
        "tick {:>2}: active = sw{}, delivered = {:>4}, dropped = {:>4}{}",
        state.tick, active, delivered, dropped, marker
    ));

    Ok((active, logs))
}

/// The failover step. Reads the standby id from the network context the
/// process carries and `alternate_value`s the active switch with it, which
/// logs the substitution. A missing switch id, context or standby id ends
/// the process in error.
pub fn fail_over(
    value: CausalEffect<SwitchId>,
    state: NetworkState,
    ctx: Option<NetworkContext>,
) -> NetworkProcess<SwitchId> {
    let switches = value
        .into_value()
        .ok_or_else(CausalityError::ValueNotAvailable)
        .and_then(|active| {
            let plan = ctx.as_ref().ok_or_else(CausalityError::MissingContext)?;
            Ok((active, read(plan, STANDBY_ID)?))
        });
    match switches {
        Ok((active, standby)) => NetworkProcess::<SwitchId>::new(
            Ok(CausalEffect::value(active)),
            state,
            ctx,
            EffectLog::new(),
        )
        .alternate_value(standby),
        Err(err) => NetworkProcess::<SwitchId>::new(Err(err), state, ctx, EffectLog::new()),
    }
}

/// Initial process routing through the primary switch, carrying `plan` as its context. A missing
/// primary id starts the process in error.
pub fn initial_process(plan: NetworkContext) -> NetworkProcess<SwitchId> {
    NetworkProcess::<SwitchId>::new(
        read(&plan, PRIMARY_ID).map(CausalEffect::value),
        NetworkState::default(),
        Some(plan),
        EffectLog::new(),
    )
}
