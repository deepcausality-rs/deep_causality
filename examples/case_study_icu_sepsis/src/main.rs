/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::run_enum::StageEnum::*;

mod run;
mod run_enum;
mod stage_one;
mod stage_two;
// mod stage_zero;

const DATA_PATH: &str = "examples/case_study_icu_sepsis/data/all/dataset.parquet";

fn main() {
    run::run(StageTwo, DATA_PATH)
}
