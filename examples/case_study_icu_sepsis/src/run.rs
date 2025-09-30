/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::run_enum::StageEnum;
use crate::{stage_one, stage_two};

pub(crate) fn run(stage: StageEnum, data_path: &str) {
    match stage {
        StageEnum::StageOne => {
            println!("Run first stage!");
            stage_one::first_stage(data_path)
        }
        StageEnum::StageTwo => {
            println!("Run second stage!");
            stage_two::second_stage(data_path);
        }
        StageEnum::All => {
            println!("Run all stages!");

            println!("Run first stage!");
            stage_one::first_stage(data_path);

            println!("Run second stage!");
            stage_two::second_stage(data_path);
        }
    }
}
