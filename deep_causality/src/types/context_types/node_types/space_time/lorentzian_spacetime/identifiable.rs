/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{Identifiable, LorentzianSpacetime};

impl Identifiable for LorentzianSpacetime {
    fn id(&self) -> u64 {
        self.id
    }
}
