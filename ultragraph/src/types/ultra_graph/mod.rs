// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::marker::PhantomData;

use deep_causality_macros::Constructor;

use crate::prelude::GraphStorage;

// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
mod graph_algorithms;
mod graph_like;
mod graph_root;
pub mod graph_storage;

#[derive(Constructor, Debug, Copy, Clone)]
pub struct UltraGraphContainer<S, T>
where
    S: GraphStorage<T>,
{
    storage: S,
    ty: PhantomData<T>,
}
