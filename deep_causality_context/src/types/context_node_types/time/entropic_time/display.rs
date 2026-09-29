/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EntropicTime, Temporal};
use core::fmt::Display;

impl Display for EntropicTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "EntropicTime: id: {}, tick_scale: {}, tick_unit: {:?}",
            self.id,
            self.time_scale(),
            self.time_unit()
        )
    }
}
