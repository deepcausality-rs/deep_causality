/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! DDoS-detector display helpers. Shared printing plumbing comes from
//! `causal_correction_examples::print_utils`.

use crate::model_types::{
    DetectorProcess, OVERLOAD_BUDGET_TICKS, SIGMA_THRESHOLD, THROTTLE_CEILING_MBPS, THROTTLE_ON,
    TRIGGER_SLOTS, ThrottleState, WINDOW_SIZE, read_real, read_ticks,
};
use causal_correction_examples::print_utils;
use deep_causality_core::CausalityError;

pub fn summary_line(label: &str, process: &DetectorProcess<ThrottleState>) {
    let st = process.state();
    let first_anomaly = match st.first_anomaly_at {
        Some(t) => format!("first anomaly at tick {t}"),
        None => "no anomaly".to_string(),
    };
    // Detection is the trigger, i.e. the tick mitigation engaged.
    let mitigated = match st.mitigated_at {
        Some(t) => format!("detected/mitigated at tick {t}"),
        None => "no mitigation".to_string(),
    };
    let outcome = match st.overload_threshold_reached_at {
        Some(t) => format!("OVERLOAD breached at tick {t}"),
        None => "within service objective".to_string(),
    };
    println!(
        "  {label}: ticks={:>2}  peak={:>6.0} Mbps  overload_ticks={:>2}  {first_anomaly}  {mitigated}  outcome={outcome}",
        st.tick, st.peak_throughput_mbps, st.overload_ticks
    );
}

/// Print one run. Reads the context facts it shows first, so a missing context or fact
/// returns the error before anything is printed.
pub fn print_section(
    label: &str,
    process: &DetectorProcess<ThrottleState>,
) -> Result<(), CausalityError> {
    let detector = process
        .context()
        .as_ref()
        .ok_or_else(CausalityError::MissingContext)?;
    let sigma_threshold = read_real(detector, SIGMA_THRESHOLD)?;
    let trigger_slots = read_ticks(detector, TRIGGER_SLOTS)?;
    let overload_budget_ticks = read_ticks(detector, OVERLOAD_BUDGET_TICKS)?;
    let throttle_ceiling_mbps = read_real(detector, THROTTLE_CEILING_MBPS)?;
    print_utils::print_section_header(label);
    let st = process.state();
    println!(
        "  ticks={}  window_size={}  sigma_threshold={}  trigger_slots={}  overload_budget={}",
        st.tick, WINDOW_SIZE, sigma_threshold, trigger_slots, overload_budget_ticks,
    );
    if let Some(t) = st.first_anomaly_at {
        println!("  first anomalous slot at tick {t}");
    }
    if let Some(t) = st.mitigated_at {
        println!(
            "  throttle engaged at tick {t} (clamped to {} Mbps)",
            throttle_ceiling_mbps
        );
    }
    print_utils::print_trajectory("throughput per tick (Mbps)", &st.throughput_history, |v| {
        format!("{v:.0}")
    });
    print_utils::print_trajectory("z-score per tick          ", &st.zscore_history, |v| {
        format!("{v:.1}")
    });
    print_utils::print_trajectory("throttle per tick         ", &st.throttle_history, |s| {
        (if *s == THROTTLE_ON { "ON" } else { "OFF" }).to_string()
    });
    match st.overload_threshold_reached_at {
        Some(t) => println!("  result: OVERLOAD breached at tick {t}"),
        None => println!("  result: within service objective"),
    }
    if let Some(v) = process.value() {
        println!(
            "  throttle at end: {}",
            if *v == THROTTLE_ON { "ON" } else { "OFF" }
        );
    }
    print_utils::print_section_footer();
    Ok(())
}
