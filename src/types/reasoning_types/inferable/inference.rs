/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use crate::prelude::*;
use crate::utils::math_utils::abs_num;

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
    type NumericValue = NumericalValue;

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

    fn conjoint_delta(&self) -> NumericalValue {
        abs_num((1.0) - self.observation)
    }

    fn is_inferable(&self) -> bool {
        if (self.observation.total_cmp(&self.threshold) == Ordering::Greater)
            && approx_equal(self.effect, self.target, 4) {
            true
        } else {
            false
        }
    }

    fn is_inverse_inferable(&self) -> bool {
        if (self.observation.total_cmp(&self.threshold) == Ordering::Less)
            && approx_equal(self.effect, self.target, 4) {
            true
        } else {
            false
        }
    }
}


impl Display for Inference
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Inference:\
               \n id: {},\n question: {},\n observation: {},\n threshold: {},\n effect: {}",
            self.id, self.question, self.observation, self.threshold, self.effect
        )
    }
}

// Because floats vary in precision, equality is not guaranteed.
// Therefore, this comparison checks for approximate equality up to a certain number
// of decimal places.
fn approx_equal(a: f64, b: f64, decimal_places: u8) -> bool {
    let factor = 10.0f64.powi(decimal_places as i32);
    let a = (a * factor).trunc();
    let b = (b * factor).trunc();
    a == b
}