/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{EuclideanSpace, Identifiable};

impl Identifiable for EuclideanSpace {
    fn id(&self) -> u64 {
        self.id
    }
}
