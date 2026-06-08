/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared test-data writers for the CDL examples, kept out of the example `main`s
//! so each example reads as the pipeline it demonstrates.

use std::fs::File;
use std::io::Write;

/// Writes the small SURD demo CSV (`s1,s2,s3,target`) and returns its path.
pub fn write_surd_test_csv() -> String {
    let csv_data =
        "s1,s2,s3,target\n1.0,2.0,3.0,1.5\n2.0,4.1,6.0,3.6\n3.0,6.2,9.0,5.4\n4.0,8.1,12.0,7.6";
    let file_path = "./test_data.csv";
    File::create(file_path)
        .unwrap()
        .write_all(csv_data.as_bytes())
        .unwrap();
    file_path.to_string()
}
