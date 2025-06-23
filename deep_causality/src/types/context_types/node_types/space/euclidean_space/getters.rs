// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::EuclideanSpace;

impl EuclideanSpace {
    pub fn coords(&self) -> [f64; 3] {
        self.coords
    }
}
