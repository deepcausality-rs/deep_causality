// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Display, Formatter};

use deep_causality_macros::Constructor;

use crate::protocols::identifiable::Identifiable;

#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Root {
    id: u64,
}

impl Identifiable for Root {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Display for Root {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Root ID: {}", self.id,)
    }
}
