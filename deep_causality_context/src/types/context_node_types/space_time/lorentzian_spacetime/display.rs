/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::LorentzianSpacetime;
use deep_causality_algebra::RealField;
use std::fmt;

impl<R: RealField + fmt::Display> fmt::Display for LorentzianSpacetime<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LorentzianSpacetime(id={}, x={:.3}, y={:.3}, z={:.3}, t={:.3} {:?})",
            self.id, self.x, self.y, self.z, self.t, self.time_scale
        )
    }
}
