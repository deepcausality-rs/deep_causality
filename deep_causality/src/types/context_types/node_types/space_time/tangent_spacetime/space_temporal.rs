// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{SpaceTemporal, TangentSpacetime};

impl SpaceTemporal<f64, f64> for TangentSpacetime {
    fn t(&self) -> &f64 {
        &self.t
    }
}
