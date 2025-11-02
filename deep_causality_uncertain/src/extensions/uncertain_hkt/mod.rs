/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Uncertain;
use deep_causality_haft::HKT;

pub struct UncertainWitness;

impl HKT for UncertainWitness {
    type Type<U> = Uncertain<U>;
}
