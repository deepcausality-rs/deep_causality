/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{Identifiable, QuaternionSpace};

impl Identifiable for QuaternionSpace {
    fn id(&self) -> u64 {
        self.id
    }
}
