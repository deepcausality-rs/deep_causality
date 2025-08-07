/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

use crate::types::context_node_types::data::Data;

impl<T> Display for Data<T>
where
    T: Debug + Default + Copy + Clone + PartialEq{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dataoid: id: {} data: {:?}", self.id, self.data)
    }
}
