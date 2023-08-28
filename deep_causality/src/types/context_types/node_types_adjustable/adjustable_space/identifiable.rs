// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::Identifiable;

use super::*;

impl<T> Identifiable for AdjustableSpace<T>
where
    T: Default + Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T> + Copy,
{
    fn id(&self) -> u64 {
        self.id
    }
}
