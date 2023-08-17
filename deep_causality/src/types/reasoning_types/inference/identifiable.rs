// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use crate::prelude::{Identifiable, IdentificationValue, Inference};

impl Identifiable for Inference
{
    fn id(&self) -> IdentificationValue {
        self.id
    }
}
