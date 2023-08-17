// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use std::fmt::{Display, Formatter};

use crate::prelude::Observation;

impl Display for Observation
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Observation {{ id: {},observation: {},observed effect: {}}}",
            self.id,
            self.observation,
            self.observed_effect
        )
    }
}
