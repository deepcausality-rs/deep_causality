/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::time_utils;

#[test]
fn test_time() {
    time_utils::time_execution(run, "run");
}

fn run() {
    println!("Hello Run")
}
