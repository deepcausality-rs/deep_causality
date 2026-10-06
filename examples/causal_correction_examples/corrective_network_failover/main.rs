/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Active/Standby Network Failover as a Corrective `intervene` Loop
//!
//! Enterprise networking pattern. A primary switch carries all traffic;
//! a standby switch sits idle, ready to take over if the primary goes
//! down. The chain forwards offered traffic through whichever switch
//! the value channel names. A scheduled failure takes the primary
//! offline mid-run. The monitor watches the per-tick delivery count.
//! When delivery drops to zero, the next tick's active switch is
//! intervened to the standby. Traffic resumes through the fallback
//! path from the very next tick.
//!
//! Two trajectories run side by side.
//!
//! * Open loop. No monitor. After the primary fails, every subsequent
//!   packet is dropped. The cumulative drop count crosses the outage
//!   threshold within a handful of seconds.
//! * Closed loop. The monitor detects the zero-delivery tick the
//!   moment it happens and fires `.alternate_value(standby)` on the
//!   chain, the standby id read from the network context. The next forward stage routes through the standby.
//!   Traffic is rerouted with at most one tick of loss.
//!
//! Same chain, same failure schedule, same offered load. The only
//! difference is whether the corrective `.intervene` wires the
//! standby into the value channel.

mod model;
pub mod model_types;
mod model_utils;

use crate::model::{fail_over, forward_traffic, initial_process};
use crate::model_types::{
    N_TICKS, NetworkContext, NetworkProcess, PRIMARY_ID, SwitchId, nominal_network_context, read,
};
use causal_correction_examples::print_utils;
use deep_causality_core::{CausalFlow, CausalityError};

fn main() -> Result<(), CausalityError> {
    println!("=== Active/Standby Network Failover as a Corrective `intervene` Loop ===\n");

    let plan =
        nominal_network_context().map_err(|err| CausalityError::GraphError(err.to_string()))?;
    let open = run_open_loop(plan.clone());
    let closed = run_closed_loop(plan);

    model_utils::print_section("Open loop (no monitor, no failover)", &open)?;
    model_utils::print_section("Closed loop (monitor + corrective failover)", &closed)?;

    println!("=== Summary ===");
    model_utils::summary_line("Open loop  ", &open);
    model_utils::summary_line("Closed loop", &closed);

    println!(
        "\nThe open-loop run loses every packet from the moment the\n\
         primary switch fails. Cumulative drops cross the outage\n\
         threshold within seconds. The closed-loop run notices the\n\
         zero-delivery tick on detection and intervenes the active\n\
         switch id from primary to standby. Traffic is back inside one\n\
         tick. The total loss is bounded to the single detection tick\n\
         instead of growing without bound."
    );

    println!("\n--- Closed-loop EffectLog (per-tick forwarding + failover event) ---");
    print_utils::print_effect_log(closed.logs());
    Ok(())
}

/// Open loop: each tick is just `forward_traffic`, run `N_TICKS` times. No monitor, no failover.
fn run_open_loop(plan: NetworkContext) -> NetworkProcess<SwitchId> {
    CausalFlow::from(initial_process(plan))
        .iterate_n(N_TICKS as usize, |tick| tick.bind(forward_traffic))
        .into_process()
}

/// Closed loop: the same tick, but each one `branch`es on the monitor — a zero-delivery tick on the
/// active primary records the failover, and the `fail_over` step `alternate_value`s the standby
/// switch into the value channel.
/// The only difference from the open loop is the `branch`. The trigger and the failover target both
/// read the network context the process carries.
fn run_closed_loop(plan: NetworkContext) -> NetworkProcess<SwitchId> {
    CausalFlow::from(initial_process(plan))
        .iterate_n(N_TICKS as usize, |tick| {
            tick.bind(forward_traffic).branch_with(
                // A predicate has no error channel. It runs only on a value, so
                // `forward_traffic` succeeded and read the same primary id from the same context.
                |active, state, ctx| {
                    let plan = ctx.expect("forward_traffic returns the network context it read");
                    let primary = read(plan, PRIMARY_ID)
                        .expect("forward_traffic read the primary id from this context");
                    state.delivered_per_tick.last() == Some(&0) && *active == primary
                },
                |hot| {
                    hot.update_state(|mut state, _active| {
                        state.failover_count += 1;
                        if state.failover_at.is_none() {
                            state.failover_at = Some(state.tick);
                        }
                        state
                    })
                    .bind(fail_over)
                },
                |cold| cold,
            )
        })
        .into_process()
}
