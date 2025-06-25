/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::time_execution;

mod run;

fn main() {
    time_execution(run::run, "main_run");
}
