// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::time::Duration;

pub fn print_duration(msg: &str, elapsed: &Duration)
{
    println!("{} took: {:?} ", msg, elapsed);
    println!();
}
