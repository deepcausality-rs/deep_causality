// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableEuclideanTime, Identifiable};

impl Identifiable for AdjustableEuclideanTime {
    fn id(&self) -> u64 {
        self.id
    }
}
