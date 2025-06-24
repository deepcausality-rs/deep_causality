// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::AdjustableEuclideanSpace;

impl AdjustableEuclideanSpace {
    pub fn coords(&self) -> [f64; 3] {
        self.coords
    }
}
