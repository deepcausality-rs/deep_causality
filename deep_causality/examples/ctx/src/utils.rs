// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::time::Duration;

pub fn print_duration(msg: &str, elapsed: &Duration)
{
    println!("{} took: {:?} ", msg, elapsed);
    println!();
}
