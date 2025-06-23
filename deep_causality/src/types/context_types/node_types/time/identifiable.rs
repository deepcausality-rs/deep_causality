// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::Identifiable;
use crate::types::context_types::node_types::time::Time;

impl Identifiable for Time {
    fn id(&self) -> u64 {
        self.id
    }
}
