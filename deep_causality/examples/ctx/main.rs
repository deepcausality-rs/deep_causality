// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

mod run;
mod bar_range;
mod rangeable;
mod dateoid;

use deep_causality::prelude::time_execution;

fn main() {
    time_execution(run::run, "main_run");
}
