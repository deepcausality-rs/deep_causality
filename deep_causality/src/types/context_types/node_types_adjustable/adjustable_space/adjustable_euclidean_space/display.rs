// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::{Display, Formatter};

use super::*;

impl Display for AdjustableEuclideanSpace {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "AdjustableEuclideanSpace(id={}, x={:.4}, y={:.4}, z={:.4})",
            self.id, self.x, self.y, self.z
        )
    }
}
