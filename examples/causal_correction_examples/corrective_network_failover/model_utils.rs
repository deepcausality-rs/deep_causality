/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Network-failover-specific display helpers. Shared printing plumbing
//! comes from `causal_correction_examples::print_utils`.

use crate::model_types::{NetworkProcess, SwitchId};
use causal_correction_examples::print_utils;

pub fn summary_line(label: &str, process: &NetworkProcess<SwitchId>) {
    let st = process.state();
    let outcome = match st.outage_threshold_reached_at {
        Some(t) => format!("OUTAGE breached at tick {t}"),
        None => "within service objective".to_string(),
    };
    let failover_text = match st.failover_at {
        Some(t) => format!("failed over at tick {t}"),
        None => "no failover".to_string(),
    };
    println!(
        "  {label}: ticks={:>2}  delivered={:>6}  dropped={:>5}  {failover_text}  outcome={outcome}",
        st.tick, st.packets_delivered_total, st.packets_dropped_total
    );
}

pub fn print_section(label: &str, process: &NetworkProcess<SwitchId>) {
    print_utils::print_section_header(label);
    let st = process.state();
    let plan = process.context().as_ref().unwrap();
    println!(
        "  ticks={}  delivered={}  dropped={}  failover_count={}  outage_threshold={}",
        st.tick,
        st.packets_delivered_total,
        st.packets_dropped_total,
        st.failover_count,
        plan.outage_drop_threshold,
    );
    if let Some(t) = st.primary_down_at {
        println!("  primary went down at tick {t}");
    }
    if let Some(t) = st.failover_at {
        println!(
            "  failover fired at tick {t} (switched to sw{})",
            plan.standby_id
        );
    }
    print_utils::print_trajectory("active switch per tick", &st.active_switch_history, |s| {
        format!("sw{s}")
    });
    print_utils::print_trajectory("delivered per tick    ", &st.delivered_per_tick, |d| {
        format!("{d}")
    });
    match st.outage_threshold_reached_at {
        Some(t) => println!("  result: OUTAGE breached at tick {t}"),
        None => println!("  result: within service objective"),
    }
    if let Some(v) = process.value() {
        println!("  active switch at end: sw{v}");
    }
    print_utils::print_section_footer();
}
