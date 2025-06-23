// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::AdjustableDiscreteTime;
use std::fmt::Display;

impl Display for AdjustableDiscreteTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AdjustableDiscreteTime: id: {}, tick_scale: {}, tick_unit: {:?}",
            self.id, self.tick_scale, self.tick_unit
        )
    }
}
