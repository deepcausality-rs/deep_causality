// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableEntropicTime, Temporal};
use std::fmt::Display;

impl Display for AdjustableEntropicTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AdjustableEntropicTime: id: {}, tick_scale: {}, tick_unit: {:?}",
            self.id, self.time_scale(), self.time_unit()
        )
    }
}
