// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

mod run;
mod protocols;
mod types;
mod workflow;
mod io;
mod config;
mod model;
mod utils;

use deep_causality::prelude::time_execution;

fn main() {
    time_execution(run::run, "main_run");
}
