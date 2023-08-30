// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Debug, Display, Formatter};

use super::*;

impl<T> Display for AdjustableData<T>
where
    T: Copy + Default + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AdjustableData: id: {} data: {:?}", self.id, self.data)
    }
}
