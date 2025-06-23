// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use std::fmt::Debug;
use std::hash::Hash;

use crate::prelude::Identifiable;
use crate::types::context_types::node_types::time::Time;

impl<T> Identifiable for Time<T>
where
    T: Copy + Clone + Hash + Eq + PartialEq + Debug,
{
    fn id(&self) -> u64 {
        self.id
    }
}
