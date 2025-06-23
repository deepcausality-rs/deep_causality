// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableLorentzianSpacetime, SpaceTemporalInterval};

impl SpaceTemporalInterval for AdjustableLorentzianSpacetime {
    fn time(&self) -> f64 {
        self.t
    }
    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
    // No need to override `interval_squared()` unless you want a custom metric for curved spacetime
}
