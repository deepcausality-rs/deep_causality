// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{Identifiable, AdjustableEuclideanSpacetime};

impl Identifiable for AdjustableEuclideanSpacetime {
    fn id(&self) -> u64 {
        self.id
    }
}
