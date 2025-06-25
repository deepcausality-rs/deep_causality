/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::prelude::{Identifiable, IdentificationValue};
use crate::types::model_types::assumption::Assumption;

impl Identifiable for Assumption {
    fn id(&self) -> IdentificationValue {
        self.id
    }
}
