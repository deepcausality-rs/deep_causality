// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableEuclideanSpacetime, SpaceTemporal};

impl SpaceTemporal<f64, f64> for AdjustableEuclideanSpacetime {
    fn t(&self) -> &f64 {
        &self.t
    }
}
