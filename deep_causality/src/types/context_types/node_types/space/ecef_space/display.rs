// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::EcefSpace;
use std::fmt;

impl fmt::Display for EcefSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EcefSpace(id={}, x={:.3}, y={:.3}, z={:.3})",
            self.id, self.x, self.y, self.z
        )
    }
}
