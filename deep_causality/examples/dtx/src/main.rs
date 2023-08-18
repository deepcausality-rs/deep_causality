use deep_causality::prelude::time_execution;

// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
mod run;
mod types;
mod config;
mod utils;
mod io;

fn main() {
    time_execution(run::run, "main_run");
}
