/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::{Display, Formatter};

use crate::prelude::Observation;

impl Display for Observation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Observation {{ id: {},observation: {},observed effect: {}}}",
            self.id, self.observation, self.observed_effect
        )
    }
}
