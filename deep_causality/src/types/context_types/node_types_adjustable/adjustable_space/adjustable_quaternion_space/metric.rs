// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableQuaternionSpace, Metric};

impl Metric<f64> for AdjustableQuaternionSpace {
    fn distance(&self, other: &Self) -> f64 {
        self.quat
            .iter()
            .zip(other.quat.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}
