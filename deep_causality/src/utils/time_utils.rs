// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

pub fn time_execution<T, F: FnOnce() -> T>(f: F, f_name: &str) -> T {
    let start = std::time::Instant::now();
    let res = f();
    println!("{} Execution took {:?} ",
             f_name.to_uppercase(),
             start.elapsed()
    );
    res
}