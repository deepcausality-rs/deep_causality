// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{Identifiable, IdentificationValue};
use crate::types::reasoning_types::assumption::Assumption;

impl Identifiable for Assumption {
    fn id(&self) -> IdentificationValue {
        self.id
    }
}
