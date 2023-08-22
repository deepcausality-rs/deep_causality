// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::Identifiable;

use super::*;

impl<T> Identifiable for AdjustableSpaceTime<T> where T: Copy + Default {
    fn id(&self) -> u64 {
        self.id
    }
}
