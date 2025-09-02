/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod adjustable;
mod datable;
mod display;
mod identifiable;

use crate::UncertainBool;

#[derive(Debug, Clone)]
pub struct UncertainBooleanData {
    id: u64,
    data: UncertainBool,
}

impl UncertainBooleanData {
    pub fn new(id: u64, data: UncertainBool) -> Self {
        Self { id, data }
    }
}
