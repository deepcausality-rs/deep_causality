/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Datable, UncertainF64, UncertainFloat64Data};

/// Implements the `Datable` trait for `UncertainF64`.
///
impl Datable for UncertainFloat64Data {
    type Data = UncertainF64;

    fn get_data(&self) -> Self::Data {
        self.data.clone()
    }

    fn set_data(&mut self, value: Self::Data) {
        self.data = value;
    }
}
