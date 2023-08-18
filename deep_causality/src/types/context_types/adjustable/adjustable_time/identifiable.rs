// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::Identifiable;
use crate::types::context_types::adjustable::adjustable_time::AdjustableTime;

impl<T> Identifiable for AdjustableTime<T> where T: Copy + Default {
    fn id(&self) -> u64 {
        self.id
    }
}