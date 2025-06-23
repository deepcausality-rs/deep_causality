// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{AdjustableEuclideanSpace, Identifiable};

impl Identifiable for AdjustableEuclideanSpace {
    fn id(&self) -> u64 {
        self.id
    }
}
