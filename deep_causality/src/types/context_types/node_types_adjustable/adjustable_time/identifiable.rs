// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::Identifiable;

use super::*;

impl Identifiable for AdjustableTime {
    fn id(&self) -> u64 {
        self.id
    }
}
