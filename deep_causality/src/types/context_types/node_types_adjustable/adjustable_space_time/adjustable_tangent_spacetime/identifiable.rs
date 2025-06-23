// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{Identifiable, AdjustableTangentSpacetime};

impl Identifiable for AdjustableTangentSpacetime {
    fn id(&self) -> u64 {
        self.id
    }
}
