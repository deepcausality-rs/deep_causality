/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod first_stage;

const DATA_PATH: &str = "examples/case_study_icu_sepsis/data/all/dataset.parquet";

fn main() {
    println!("Run first stage!");
    first_stage::first_stage(DATA_PATH);
}
