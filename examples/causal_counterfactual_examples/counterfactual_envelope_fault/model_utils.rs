/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Envelope-fault-specific display helpers. Shared printing plumbing
//! comes from `causal_counterfactual_examples::print_utils`.

use crate::model_types::{FlightProcess, Verdict};
use causal_counterfactual_examples::print_utils;

pub fn print_section(label: &str, process: &FlightProcess<Verdict>) {
    print_utils::print_section_header(label);
    match process.value() {
        Some(v) => println!(
            "  verdict: {v:?}  (risk={:.3}, est_airspeed={:.0} kn)",
            process.state().risk,
            process.state().estimate_airspeed_kn,
        ),
        None => println!("  verdict: <none>  (error: {:?})", process.error()),
    }
    println!("  EffectLog:");
    print_utils::print_effect_log_indented(process.logs(), "    ");
    print_utils::print_section_footer();
}
