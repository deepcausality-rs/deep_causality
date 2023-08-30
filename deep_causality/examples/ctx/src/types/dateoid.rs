// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::{Display, Formatter};

use deep_causality::prelude::{Datable, Identifiable};

use crate::protocols::rangeable::Rangeable;
use crate::types::bar_range::BarRange;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct CustomData {
    id: u64,
    data_range: BarRange,
}

impl CustomData {
    pub fn new(id: u64, data_range: BarRange) -> Self {
        Self { id, data_range }
    }
}

impl Datable for CustomData {}

impl Identifiable for CustomData {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Rangeable for CustomData {
    fn data_range(&self) -> BarRange {
        self.data_range
    }
}

impl Display for CustomData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {} range: {}", self.id, self.data_range)
    }
}
