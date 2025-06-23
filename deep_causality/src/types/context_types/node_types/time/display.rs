// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::{Debug, Display};
use std::hash::Hash;
use crate::types::context_types::node_types::time::Time;

impl<T> Display for Time<T>
where
T: Copy + Clone + Hash + Eq + PartialEq +Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tempoid: id: {}, time_scale: {}, time_unit: {:?}",
            self.id, self.time_scale, self.time_unit
        )
    }
}
