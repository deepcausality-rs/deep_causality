// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::QuaternionSpace;

impl std::fmt::Display for QuaternionSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "QuaternionSpace(id={}, w={:.3}, x={:.3}, y={:.3}, z={:.3})",
            self.id, self.quat[0], self.quat[1], self.quat[2], self.quat[3]
        )
    }
}