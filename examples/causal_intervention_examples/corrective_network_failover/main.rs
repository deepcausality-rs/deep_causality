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
//!   moment it happens and fires `.intervene(STANDBY_SWITCH)` on the
//!   chain. The next forward stage routes through the standby.
//!   Traffic is rerouted with at most one tick of loss.
//!
//! Same chain, same failure schedule, same offered load. The only
//! difference is whether the corrective `.intervene` wires the
//! standby into the value channel.

mod model;
pub mod model_types;
mod model_utils;

use crate::model::{forward_traffic, initial_process};
use crate::model_types::{N_TICKS, NetworkProcess, STANDBY_SWITCH, SwitchId};
use causal_intervention_examples::print_utils;
use deep_causality_core::{EffectValue, Intervenable};

fn main() {
    println!("=== Active/Standby Network Failover as a Corrective `intervene` Loop ===\n");

    let open = run_open_loop();
    let closed = run_closed_loop();

    model_utils::print_section("Open loop (no monitor, no failover)", &open);
    model_utils::print_section("Closed loop (monitor + corrective failover)", &closed);

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
    print_utils::print_effect_log(&closed.logs);
}

fn run_open_loop() -> NetworkProcess<SwitchId> {
    let mut process = initial_process();
    for _ in 0..N_TICKS {
        process = process.bind(forward_traffic);
    }
    process
}

fn run_closed_loop() -> NetworkProcess<SwitchId> {
    let mut process = initial_process();
    for _ in 0..N_TICKS {
        process = process.bind(forward_traffic);

        let plan = process.context.clone().expect("NetworkPlan present");
        let active = match &process.value {
            EffectValue::Value(v) => *v,
            _ => continue,
        };
        let last_delivered = process
            .state
            .delivered_per_tick
            .last()
            .copied()
            .unwrap_or(0);

        if last_delivered == 0 && active == plan.primary_id {
            process.state.failover_count += 1;
            if process.state.failover_at.is_none() {
                process.state.failover_at = Some(process.state.tick);
            }
            process = process.intervene(STANDBY_SWITCH);
        }
    }
    process
}
