// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::AdjustableNedSpace;
use std::fmt;

impl fmt::Display for AdjustableNedSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AdjustableNedSpace(id={}, N={:.3}, E={:.3}, D={:.3})",
            self.id, self.north, self.east, self.down
        )
    }
}
