// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{EuclideanSpacetime, Identifiable};

impl Identifiable for EuclideanSpacetime {
    fn id(&self) -> u64 {
        self.id
    }
}
