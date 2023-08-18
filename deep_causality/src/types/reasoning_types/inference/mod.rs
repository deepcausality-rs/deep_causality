// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

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
