// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::Display;

use deep_causality_macros::Constructor;

use crate::prelude::{Identifiable, Temporable, TimeScale};

#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Tempoid {
    id: u64,
    time_scale: TimeScale,
    time_unit: u32,
}

impl Identifiable for Tempoid {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Temporable for Tempoid {
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> u32 {
        self.time_unit
    }
}

impl Display for Tempoid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tempoid: id: {}, time_scale: {}, time_unit: {}",
            self.id, self.time_scale, self.time_unit
        )
    }
}
