/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EntropicTime, Identifiable};

impl Identifiable for EntropicTime {
    fn id(&self) -> u64 {
        self.id
    }
}
