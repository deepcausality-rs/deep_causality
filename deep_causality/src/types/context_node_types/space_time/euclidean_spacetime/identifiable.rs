/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EuclideanSpacetime, Identifiable};

impl Identifiable for EuclideanSpacetime {
    fn id(&self) -> u64 {
        self.id
    }
}
