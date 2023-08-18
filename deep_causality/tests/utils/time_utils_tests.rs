// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::time_execution;

#[test]
fn test_time()
{
    time_execution(run, "run");
}

fn run() {
    println!("Hello Run")
}