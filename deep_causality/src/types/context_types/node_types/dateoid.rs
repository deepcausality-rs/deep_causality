// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Display, Formatter};

use deep_causality_macros::{Constructor, Getters};

use crate::prelude::{Datable, Identifiable};

#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Dataoid {
    #[getter(name = data_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    data: i32,
}

impl Datable for Dataoid {}

impl Identifiable for Dataoid {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Display for Dataoid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dataoid: id: {} data: {}", self.id, self.data)
    }
}
