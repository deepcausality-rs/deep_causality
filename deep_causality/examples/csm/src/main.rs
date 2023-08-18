// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::time_execution;

mod run;
mod utils_actions;
mod utils_states;
mod utils_data;

fn main() {
    time_execution(run::run, "main_run");
}
