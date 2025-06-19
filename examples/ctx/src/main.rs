// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::time_execution;

mod config;
mod io;
mod model;
mod protocols;
mod run;
mod types;
mod utils;
mod workflow;

fn main() {
    time_execution(run::run, "main_run");
}
