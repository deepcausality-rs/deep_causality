// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use std::fmt;
use crate::prelude::NedSpace;

impl fmt::Display for NedSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NedSpace(id={}, N={:.3}, E={:.3}, D={:.3})",
            self.id, self.north, self.east, self.down
        )
    }
}

