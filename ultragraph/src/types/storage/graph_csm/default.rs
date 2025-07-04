/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CsmGraph;

impl<N, W> Default for CsmGraph<N, W>
where
    W: Default,
{
    fn default() -> Self {
        Self::new()
    }
}
