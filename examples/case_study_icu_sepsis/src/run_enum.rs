/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::Display;

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

//  The tests only exists to silence clippy warnings on unused code / non constructed values
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_for_stage_enum() {
        assert_eq!(format!("{}", StageEnum::StageOne), "StageOne");
        assert_eq!(format!("{}", StageEnum::StageTwo), "StageTwo");
        assert_eq!(format!("{}", StageEnum::All), "All");
    }

    #[test]
    fn test_stage_enum_variants() {
        let stage_one = StageEnum::StageOne;
        assert!(matches!(stage_one, StageEnum::StageOne));

        let stage_two = StageEnum::StageTwo;
        assert!(matches!(stage_two, StageEnum::StageTwo));

        let all = StageEnum::All;
        assert!(matches!(all, StageEnum::All));
    }
}
