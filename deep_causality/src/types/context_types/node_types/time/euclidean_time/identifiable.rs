// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{EuclideanTime, Identifiable};

impl Identifiable for EuclideanTime {
    fn id(&self) -> u64 {
        self.id
    }
}
