// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableEuclideanSpacetime, Coordinate};

impl Coordinate<f64> for AdjustableEuclideanSpacetime {
    fn dimension(&self) -> usize {
        3
    }

    fn coordinate(&self, index: usize) -> &f64 {
        &self.coords[index]
    }
}
