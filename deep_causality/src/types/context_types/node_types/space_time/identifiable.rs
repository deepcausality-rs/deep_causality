// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::{Add, Mul, Sub};

use crate::prelude::Identifiable;
use crate::types::context_types::node_types::space_time::SpaceTempoid;

impl<T> Identifiable for SpaceTempoid<T>
where
    T: Default + Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T>,
{
    fn id(&self) -> u64 {
        self.id
    }
}
