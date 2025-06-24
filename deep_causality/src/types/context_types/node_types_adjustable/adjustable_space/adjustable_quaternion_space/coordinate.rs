// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableQuaternionSpace, Coordinate};

impl Coordinate<f64> for AdjustableQuaternionSpace {
    fn dimension(&self) -> usize {
        4
    }

    fn coordinate(&self, index: usize) -> &f64 {
        &self.quat[index]
    }
}
