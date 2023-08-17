// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::{IdentificationValue, NumericalValue};

mod identifiable;
mod observable;
mod display;


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Observation {
    id: IdentificationValue,
    observation: NumericalValue,
    observed_effect: NumericalValue,
}

impl Observation
{
    pub fn new(id: IdentificationValue, observation: NumericalValue, observed_effect: NumericalValue) -> Self {
        Self { id, observation, observed_effect }
    }
}
