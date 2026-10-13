/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fast-charge display helpers. Shared printing plumbing comes from
//! `causal_counterfactual_examples::print_utils`.

use crate::model::charge_current_a;
use crate::model_config::RUN_MINUTES;
use crate::model_types::{CoolingContext, FloatType};
use causal_counterfactual_examples::print_utils;
use deep_causality_core::PropagatingProcess;

/// Print a run's end point, the current the charger would draw next, and its audit log.
pub fn print_run(label: &str, run: &PropagatingProcess<FloatType, FloatType, CoolingContext>) {
    print_utils::print_section_header(label);
    match run.value() {
        Some(cell_c) => println!(
            "  after {RUN_MINUTES} minutes: {cell_c:.1} °C, {:.1} % charged, charging at {:.1} A",
            run.state(),
            charge_current_a(*cell_c, *run.state())
        ),
        None => println!(
            "  after {RUN_MINUTES} minutes: <none>  (error: {:?})",
            run.error()
        ),
    }
    println!("{}", run.logs().to_string().trim_end());
    print_utils::print_section_footer();
}
