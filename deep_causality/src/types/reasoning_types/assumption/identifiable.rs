// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use crate::prelude::{Identifiable, IdentificationValue};
use crate::types::reasoning_types::assumption::Assumption;

impl Identifiable for Assumption
{
    fn id(&self) -> IdentificationValue {
        self.id
    }
}
