/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{NumericalValue, Observable, Observation};

impl Observable for Observation {
    fn observation(&self) -> NumericalValue {
        self.observation
    }

    fn observed_effect(&self) -> NumericalValue {
        self.observed_effect
    }
}
