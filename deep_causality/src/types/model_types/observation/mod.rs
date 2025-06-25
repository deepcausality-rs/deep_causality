/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_macros::Constructor;

use crate::prelude::{IdentificationValue, NumericalValue};

mod display;
mod identifiable;
mod observable;

#[derive(Constructor, Debug, Clone, PartialEq, PartialOrd)]
pub struct Observation {
    id: IdentificationValue,
    observation: NumericalValue,
    observed_effect: NumericalValue,
}
