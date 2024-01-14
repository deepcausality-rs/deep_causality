// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::time_execution;

mod run;
mod utils_actions;
mod utils_data;
mod utils_states;

fn main() {
    time_execution(run::run, "main_run");
}
