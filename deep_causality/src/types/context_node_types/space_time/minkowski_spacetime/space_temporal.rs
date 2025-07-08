/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{MinkowskiSpacetime, SpaceTemporal};

impl SpaceTemporal<f64, f64> for MinkowskiSpacetime {
    fn t(&self) -> &f64 {
        &self.t
    }
}
