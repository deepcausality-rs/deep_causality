/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::time_utils;

mod run;
mod utils_actions;
mod utils_data;
mod utils_states;

fn main() {
    time_utils::time_execution(run::run, "main_run");
}
