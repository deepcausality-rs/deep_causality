// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Debug, Display, Formatter};

use crate::types::context_types::node_types::data::Data;

impl<T> Display for Data<T>
where
    T: Copy + Default + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dataoid: id: {} data: {:?}", self.id, self.data)
    }
}
