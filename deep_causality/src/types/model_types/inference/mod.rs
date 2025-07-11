/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_macros::Constructor;

use crate::{DescriptionValue, IdentificationValue, NumericalValue};

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
