// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::SpaceTemporal;

use super::*;

impl<T> SpaceTemporal for AdjustableSpaceTime<T>
where
    T: Copy + Default,
{
    fn t(&self) -> u64 {
        self.time_unit as u64
    }
}
