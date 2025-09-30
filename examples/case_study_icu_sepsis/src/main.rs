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

#[allow(dead_code)]
const FULL_DATA_PATH: &str = "examples/case_study_icu_sepsis/data/all/dataset.parquet";
#[allow(dead_code)]
const SEPS_TRUE_PATH: &str = "examples/case_study_icu_sepsis/data/seperated/seps_true.parquet";
#[allow(dead_code)]
const SEPS_FALSE_PATH: &str = "examples/case_study_icu_sepsis/data/seperated/seps_false.parquet";

fn main() {
    run::run(StageTwo, SEPS_FALSE_PATH)
}
