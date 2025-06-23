// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableLorentzianSpacetime, SpaceTemporal};

impl SpaceTemporal<f64> for AdjustableLorentzianSpacetime {
    fn t(&self) -> &f64 {
        &self.t
    }
}