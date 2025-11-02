/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Uncertain;
use deep_causality_haft::HKT;

pub struct MaybeUncertainWitness {}

impl HKT for MaybeUncertainWitness {
    type Type<T> = Uncertain<T>;
}
