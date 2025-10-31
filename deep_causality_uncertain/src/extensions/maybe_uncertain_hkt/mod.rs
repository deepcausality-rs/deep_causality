/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::HKT;

use crate::MaybeUncertain;

pub struct MaybeUncertainWitness;

impl HKT for MaybeUncertainWitness {
    type Type<U> = MaybeUncertain<U>;
}
