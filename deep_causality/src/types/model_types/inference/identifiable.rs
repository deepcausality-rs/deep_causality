// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{Identifiable, IdentificationValue, Inference};

impl Identifiable for Inference {
    fn id(&self) -> IdentificationValue {
        self.id
    }
}
