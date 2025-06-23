// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableEuclideanSpacetime, Metric};

impl Metric<f64> for AdjustableEuclideanSpacetime {
    fn distance(&self, other: &Self) -> f64 {
        self.coords.iter()
            .zip(other.coords.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}