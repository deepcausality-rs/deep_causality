// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::LorentzianSpacetime;
use std::fmt;

impl fmt::Display for LorentzianSpacetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LorentzianSpacetime(id={}, x={:.3}, y={:.3}, z={:.3}, t={:.3} {:?})",
            self.id, self.x, self.y, self.z, self.t, self.time_scale
        )
    }
}
