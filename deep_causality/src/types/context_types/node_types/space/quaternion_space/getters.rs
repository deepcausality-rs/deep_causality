// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::QuaternionSpace;

impl QuaternionSpace {

    pub fn quat(&self) -> [f64; 4] {
        self.quat
    }
}