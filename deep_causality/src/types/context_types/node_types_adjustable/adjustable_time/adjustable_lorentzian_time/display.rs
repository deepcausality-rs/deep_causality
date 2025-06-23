// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::AdjustableLorentzianTime;
use std::fmt::Display;

impl Display for AdjustableLorentzianTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AdjustableLorentzianTime: id: {}, time_scale: {}, time_unit: {:?}",
            self.id, self.time_scale, self.time_unit
        )
    }
}
