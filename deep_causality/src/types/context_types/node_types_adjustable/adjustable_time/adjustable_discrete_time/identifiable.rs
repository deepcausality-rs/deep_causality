// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableDiscreteTime, Identifiable};

impl Identifiable for AdjustableDiscreteTime {
    fn id(&self) -> u64 {
        self.id
    }
}
