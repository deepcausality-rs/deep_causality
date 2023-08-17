// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::{DescriptionValue, IdentificationValue, NumericalValue};

mod identifiable;
mod inferable;
mod display;


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Inference
{
    id: IdentificationValue,
    question: DescriptionValue,
    observation: NumericalValue,
    threshold: NumericalValue,
    effect: NumericalValue,
    target: NumericalValue,
}

impl Inference
{
    pub fn new(
        id: IdentificationValue, question: DescriptionValue, observation: NumericalValue,
        threshold: NumericalValue, effect: NumericalValue, target: NumericalValue,
    )
        -> Self
    {
        Self { id, question, observation, threshold, effect, target }
    }
}
