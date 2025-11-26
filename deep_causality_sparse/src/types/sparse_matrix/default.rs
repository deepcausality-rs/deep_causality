/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CsrMatrix;

impl<T> Default for CsrMatrix<T> {
    fn default() -> Self {
        Self::new()
    }
}
