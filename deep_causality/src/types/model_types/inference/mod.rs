// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use deep_causality_macros::Constructor;

use crate::prelude::{DescriptionValue, IdentificationValue, NumericalValue};

mod display;
mod identifiable;
mod inferable;

#[derive(Constructor, Debug, Clone, PartialEq, PartialOrd)]
pub struct Inference {
    id: IdentificationValue,
    question: DescriptionValue,
    observation: NumericalValue,
    threshold: NumericalValue,
    effect: NumericalValue,
    target: NumericalValue,
}
