/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{Identifiable, SymbolicTime};

impl Identifiable for SymbolicTime {
    fn id(&self) -> u64 {
        self.id
    }
}
