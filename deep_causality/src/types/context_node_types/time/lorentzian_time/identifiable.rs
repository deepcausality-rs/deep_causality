/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{Identifiable, LorentzianTime};

impl Identifiable for LorentzianTime {
    fn id(&self) -> u64 {
        self.id
    }
}
