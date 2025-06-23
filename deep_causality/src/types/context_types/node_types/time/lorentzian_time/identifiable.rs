// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{Identifiable, LorentzianTime};

impl Identifiable for LorentzianTime {
    fn id(&self) -> u64 {
        self.id
    }
}
