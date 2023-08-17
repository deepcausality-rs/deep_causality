// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use crate::prelude::{Identifiable, IdentificationValue, Observation};

impl Identifiable for Observation
{
    fn id(&self) -> IdentificationValue {
        self.id
    }
}
