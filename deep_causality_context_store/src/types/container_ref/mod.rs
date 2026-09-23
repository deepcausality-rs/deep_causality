/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextId;

/// Names a container inside a commit: one the store holds, or one the same commit creates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerRef {
    /// A container the store holds under this identifier.
    Held(ContextId),
    /// The container made by the commit's `n`-th `ContextWrite::CreateContext`, counting from 0.
    /// It names only a container an earlier write of the same commit created.
    Created(usize),
}
