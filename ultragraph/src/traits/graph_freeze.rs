/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CsmGraph, EdgeKey};

pub trait Freezable<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    fn freeze(self) -> CsmGraph<N, W>;
}
