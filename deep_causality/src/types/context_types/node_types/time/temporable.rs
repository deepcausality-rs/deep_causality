// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use std::fmt::Debug;
use std::hash::Hash;

use crate::prelude::TimeScale;
use crate::traits::contextuable::temporal::Temporal;
use crate::types::context_types::node_types::time::Time;

impl<T> Temporal<T> for Time<T>
where
    T: Copy + Clone + Hash + Eq + PartialEq + Debug,
{
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> &T {
        &self.time_unit
    }
}
