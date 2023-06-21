/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use std::fmt::{Display, Formatter};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Inference {
    id: IdentificationValue,
    question: DescriptionValue,
    observation: NumericalValue,
    threshold: NumericalValue,
    effect: NumericalValue,
    target: NumericalValue,
}

impl Inference {
    pub fn new(id: IdentificationValue, question: DescriptionValue, observation: NumericalValue,
               threshold: NumericalValue, effect: NumericalValue, target: NumericalValue, ) -> Self {
        Self { id, question, observation, threshold, effect, target }
    }
}


impl Identifiable for Inference {
    fn id(&self) -> IdentificationValue {
        self.id
    }
}

impl Inferable for Inference
{
    fn question(&self) -> DescriptionValue {
        self.question.to_string()
    }

    fn observation(&self) -> NumericalValue {
        self.observation
    }

    fn threshold(&self) -> NumericalValue {
        self.threshold
    }

    fn effect(&self) -> NumericalValue {
        self.effect
    }

    fn target(&self) -> NumericalValue {
        self.target
    }
}


impl Display for Inference
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Inference: \n id: {},\n question: {},\n observation: {},\n threshold: {},\n effect: {}",
            self.id, self.question, self.observation, self.threshold, self.effect
        )
    }
}
