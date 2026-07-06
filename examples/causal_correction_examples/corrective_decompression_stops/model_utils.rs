/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Decompression-specific display helpers. Shared printing plumbing
//! comes from `causal_correction_examples::print_utils`.

use crate::model_types::{DiveProcess, FloatType};
use causal_correction_examples::print_utils;

pub fn summary_line(label: &str, process: &DiveProcess<FloatType>) {
    let st = process.state();
    let outcome = match st.dcs_at {
        Some(t) => format!("DCS RISK at tick {t}"),
        None => "surfaced safely".to_string(),
    };
    println!(
        "  {label}: ticks={:>2}  stops={:>2}  final_depth={:>4.1} m  max_ratio={:>4.2}  outcome={outcome}",
        st.tick, st.stop_count, st.depth_m, st.max_ratio_observed,
    );
}

pub fn print_section(label: &str, process: &DiveProcess<FloatType>) {
    print_utils::print_section_header(label);
    let st = process.state();
    let cfg = process.context().as_ref().unwrap();
    println!(
        "  ticks={}  stops={}  final_depth={:.1} m  max_ratio={:.2}  dcs_threshold={:.2}",
        st.tick, st.stop_count, st.depth_m, st.max_ratio_observed, cfg.dcs_ratio_threshold
    );
    print_utils::print_trajectory("depth (m)", &st.depth_trajectory, |x| format!("{x:.1}"));
    print_utils::print_trajectory("ratio    ", &st.ratio_trajectory, |x| format!("{x:.2}"));
    match st.dcs_at {
        Some(t) => println!("  result: DCS RISK at tick {t}"),
        None => println!("  result: surfaced safely"),
    }
    if let Some(v) = process.value() {
        println!("  next ascent command: {v:.1} m");
    }
    print_utils::print_section_footer();
}
