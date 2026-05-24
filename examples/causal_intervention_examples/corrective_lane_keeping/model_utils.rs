/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lane-keeping-specific display helpers. Shared printing plumbing
//! comes from `causal_intervention_examples::print_utils`.

use crate::model_types::{FloatType, LaneProcess};
use causal_intervention_examples::print_utils;
use deep_causality_core::EffectValue;

pub fn summary_line(label: &str, process: &LaneProcess<FloatType>) {
    let st = &process.state;
    let outcome = match st.catastrophic_at {
        Some(t) => format!("OFF-ROAD at tick {t}"),
        None => "stayed in lane".to_string(),
    };
    println!(
        "  {label}: ticks={:>2}  corrections={:>2}  max_|offset|={:>5.2} m  outcome={outcome}",
        st.tick,
        st.correction_count,
        st.max_offset_observed.abs(),
    );
}

pub fn print_section(label: &str, process: &LaneProcess<FloatType>) {
    print_utils::print_section_header(label);
    let st = &process.state;
    let cfg = process.context.as_ref().unwrap();
    println!(
        "  ticks={}  corrections={}  max_|offset|={:.2} m  lane_half_width={:.2} m",
        st.tick,
        st.correction_count,
        st.max_offset_observed.abs(),
        cfg.lane_half_width,
    );
    print_utils::print_trajectory("trajectory (m)", &st.trajectory, |x| format!("{x:+.2}"));
    match st.catastrophic_at {
        Some(t) => println!("  result: OFF-ROAD at tick {t}"),
        None => println!("  result: stayed in lane"),
    }
    if let EffectValue::Value(v) = process.value {
        println!("  final offset: {v:+.2} m");
    }
    print_utils::print_section_footer();
}
