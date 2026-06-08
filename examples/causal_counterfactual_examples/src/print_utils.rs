/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared printing helpers used across the counterfactual and corrective
//! intervention examples. Every helper takes generic or primitive
//! arguments and knows nothing about any specific example.

use deep_causality_core::EffectLog;

/// Print a section header in the standard `--- {label} ---` format.
pub fn print_section_header(label: &str) {
    println!("--- {label} ---");
}

/// Trailing blank line for visual separation between sections.
pub fn print_section_footer() {
    println!();
}

/// Print a labelled, comma-separated trajectory inside square brackets.
/// The formatter closure decides each element's display.
pub fn print_trajectory<T>(label: &str, values: &[T], formatter: impl Fn(&T) -> String) {
    print!("  {label}: [");
    for (i, x) in values.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{}", formatter(x));
    }
    println!("]");
}

/// Print an `EffectLog` line by line, indented with the given prefix.
pub fn print_effect_log_indented(logs: &EffectLog, indent: &str) {
    let text = format!("{logs:?}");
    for line in text.split(',').map(|s| s.trim()) {
        if !line.is_empty() {
            println!("{indent}{line}");
        }
    }
}

/// Print an `EffectLog` with the standard two-space indent.
pub fn print_effect_log(logs: &EffectLog) {
    print_effect_log_indented(logs, "  ");
}
