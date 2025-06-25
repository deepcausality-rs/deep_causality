/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::time_execution;

#[test]
fn test_time() {
    time_execution(run, "run");
}

fn run() {
    println!("Hello Run")
}
