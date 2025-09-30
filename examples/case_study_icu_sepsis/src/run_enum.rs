/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::Display;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum StageEnum {
    StageOne,
    StageTwo,
    All,
}

impl Display for StageEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageEnum::StageOne => write!(f, "StageOne"),
            StageEnum::StageTwo => write!(f, "StageTwo"),
            StageEnum::All => write!(f, "All"),
        }
    }
}
