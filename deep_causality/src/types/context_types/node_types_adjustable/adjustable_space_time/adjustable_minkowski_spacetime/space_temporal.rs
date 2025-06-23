// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableMinkowskiSpacetime, SpaceTemporal};

impl SpaceTemporal<f64> for AdjustableMinkowskiSpacetime {
    fn t(&self) -> &f64 {
        &self.t
    }
}
