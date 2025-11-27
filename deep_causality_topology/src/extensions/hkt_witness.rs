/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Chain, Topology};
use deep_causality_haft::HKT;

// Define Witness Types for HKT
pub struct ChainWitness;

impl HKT for ChainWitness {
    type Type<T> = Chain<T>;
}

pub struct CausalTopologyWitness;

impl HKT for CausalTopologyWitness {
    type Type<T> = Topology<T>;
}
