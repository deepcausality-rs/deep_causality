/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Glucose-pump-specific display helpers. Shared printing plumbing
//! comes from `causal_correction_examples::print_utils`.

use crate::model_types::{FloatType, KETOACIDOSIS_THRESHOLD, PumpProcess, TICK_MINUTES, read};
use causal_correction_examples::print_utils;
use deep_causality_core::CausalityError;

pub fn summary_line(label: &str, process: &PumpProcess<FloatType>) {
    let st = process.state();
    let outcome = match st.ketoacidosis_at {
        Some(t) => format!("KETOACIDOSIS at tick {t} (t = {} min)", t * TICK_MINUTES),
        None => "stayed in safe range".to_string(),
    };
    println!(
        "  {label}: ticks={:>2}  boluses={:>2}  total_insulin={:>4.1} U  max_glucose={:>5.1} mg/dL  outcome={outcome}",
        st.tick, st.bolus_count, st.total_insulin_units, st.max_glucose_observed,
    );
}

/// Print one run. Reads the context facts it shows first, so a missing context or fact
/// returns the error before anything is printed.
pub fn print_section(label: &str, process: &PumpProcess<FloatType>) -> Result<(), CausalityError> {
    let pump = process
        .context()
        .as_ref()
        .ok_or_else(CausalityError::MissingContext)?;
    let ketoacidosis_threshold = read(pump, KETOACIDOSIS_THRESHOLD)?;
    print_utils::print_section_header(label);
    let st = process.state();
    println!(
        "  ticks={}  boluses={}  total_insulin={:.1} U  max_glucose={:.1} mg/dL  ketoacidosis_threshold={:.0} mg/dL",
        st.tick,
        st.bolus_count,
        st.total_insulin_units,
        st.max_glucose_observed,
        ketoacidosis_threshold,
    );
    print_utils::print_trajectory("trajectory (mg/dL)", &st.trajectory, |x| format!("{x:.0}"));
    match st.ketoacidosis_at {
        Some(t) => println!("  result: KETOACIDOSIS at tick {t}"),
        None => println!("  result: stayed in safe range"),
    }
    if let Some(v) = process.value() {
        println!("  final glucose: {v:.1} mg/dL");
    }
    print_utils::print_section_footer();
    Ok(())
}
