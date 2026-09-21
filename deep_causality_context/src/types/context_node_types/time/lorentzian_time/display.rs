/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::LorentzianTime;
use deep_causality_algebra::RealField;
use std::fmt::{Debug, Display};

impl<R: RealField + Display + Debug> Display for LorentzianTime<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LorentzianTime: id: {}, time_scale: {}, time_unit: {:?}",
            self.id, self.time_scale, self.time_unit
        )
    }
}
