// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Display, Formatter};

use crate::types::context_types::adjustable::adjustable_data::AdjustableData;

impl<T> Display for AdjustableData<T> where T: Copy + Default + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AdjustableData: id: {} data: {}", self.id, self.data)
    }
}
