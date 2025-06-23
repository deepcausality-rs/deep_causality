// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::Identifiable;

use super::*;

impl<T> Identifiable for AdjustableTime<T>
where
    T: Debug
        + Default
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Mul<T, Output = T>
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + PartialOrd,
{
    fn id(&self) -> u64 {
        self.id
    }
}
