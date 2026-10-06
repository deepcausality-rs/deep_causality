/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Aneurysm-specific display helpers. Shared printing plumbing comes
//! from `causal_counterfactual_examples::print_utils`.

use crate::model_types::{CycleSummary, PatientProcess};
use causal_counterfactual_examples::print_utils;

pub fn print_process(label: &str, process: &PatientProcess<CycleSummary>) {
    print_utils::print_section_header(label);
    match process.value() {
        Some(s) => {
            println!(
                "  cycles={:>2}  peak_WSS={:>5.2} Pa  final_fatigue={:>5.1}%  ruptured={}",
                s.cycles_run,
                s.peak_wss,
                s.final_fatigue * 100.0,
                if s.ruptured { "YES" } else { "no" },
            );
        }
        None => println!("  (no value)"),
    }
    print_utils::print_section_footer();
}

pub fn print_audit_trail(process: &PatientProcess<CycleSummary>) {
    println!("--- Audit trail of the surgical counterfactual ---");
    print_utils::print_effect_log(process.logs());
    println!(
        "\nThe log shows the intervention firing on WSS, not on BP. \"We\n\
         attenuated stress directly\" is now a provable claim about this\n\
         chain rather than an assertion in the README. Compare with the\n\
         medication arm's log to see the contrast in which variable was\n\
         the target of the intervention.\n"
    );
}
