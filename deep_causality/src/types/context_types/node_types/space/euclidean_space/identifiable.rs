// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{EuclideanSpace, Identifiable};

impl Identifiable for EuclideanSpace {
    fn id(&self) -> u64 {
        self.id
    }
}
