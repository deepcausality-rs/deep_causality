// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::time_execution;

mod run;
mod protocols;
mod types;
mod workflow;
mod io;
mod config;
mod model;
mod utils;

fn main() {
    time_execution(run::run, "main_run");
}
