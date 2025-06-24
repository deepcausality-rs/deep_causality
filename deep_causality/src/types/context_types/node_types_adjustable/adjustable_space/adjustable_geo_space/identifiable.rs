// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{AdjustableGeoSpace, Identifiable};

impl Identifiable for AdjustableGeoSpace {
    fn id(&self) -> u64 {
        self.id
    }
}
