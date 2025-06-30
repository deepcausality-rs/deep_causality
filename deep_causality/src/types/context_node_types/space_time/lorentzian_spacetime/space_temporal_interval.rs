/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{LorentzianSpacetime, SpaceTemporalInterval};

impl SpaceTemporalInterval for LorentzianSpacetime {
    fn time(&self) -> f64 {
        self.t
    }
    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
    // No need to override `interval_squared()` unless you want a custom metric for curved spacetime
}
