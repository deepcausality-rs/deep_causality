/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::prelude::{Identifiable, IdentificationValue, Observation};

impl Identifiable for Observation {
    fn id(&self) -> IdentificationValue {
        self.id
    }
}
