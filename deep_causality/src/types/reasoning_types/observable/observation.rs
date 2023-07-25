// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use std::fmt::{Display, Formatter};

use crate::prelude::*;
use crate::protocols::observable::Observable;

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


impl Identifiable for Observation
{
    fn id(&self) -> IdentificationValue {
        self.id
    }
}


impl Observable for Observation
{
    fn observation(&self) -> NumericalValue {
        self.observation
    }

    fn observed_effect(&self) -> NumericalValue {
        self.observed_effect
    }
}

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
