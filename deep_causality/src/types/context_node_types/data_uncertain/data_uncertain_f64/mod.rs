/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod adjustable;
mod datable;
mod display;
mod identifiable;

use crate::UncertainF64;

#[derive(Debug, Clone)]
pub struct UncertainFloat64Data {
    id: u64,
    data: UncertainF64,
}

impl UncertainFloat64Data {
    pub fn new(id: u64, data: UncertainF64) -> Self {
        Self { id, data }
    }
}
