// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableMinkowskiSpacetime, Identifiable};

impl Identifiable for AdjustableMinkowskiSpacetime {
    fn id(&self) -> u64 {
        self.id
    }
}
