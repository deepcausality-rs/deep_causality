/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
use std::fmt::{Display, Formatter};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Observation {
    id: IdentificationValue,
    observed_trigger: NumericalValue,
    observed_factor: NumericalValue,
}

impl Observation
{
    pub fn new(id: IdentificationValue, observed_trigger: NumericalValue, observed_factor: NumericalValue) -> Self {
        Self { id, observed_trigger, observed_factor }
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
        self.observed_trigger
    }

    fn effect_observed(
        &self,
        target_threshold: NumericalValue,
        target_effect: NumericalValue,
    ) -> bool {
        return if (self.observed_trigger >= target_threshold)
            && (self.observed_factor == target_effect)
        {
            true
        } else {
            false
        };
    }
}

impl Display for Observation
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Observation {{\n id: {},\n observation: {},\n }}",
            self.id, self.observed_trigger,
        )
    }
}
