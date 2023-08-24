// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::marker::PhantomData;

use deep_causality_macros::{Constructor, Getters};

mod adjustable;
mod display;
mod identifiable;
mod spatial;

#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableSpace<T>
where
    T: Copy + Default,
{
    id: u64,
    x: T,
    y: T,
    z: T,
    ty: PhantomData<T>, // Need to bind T
}
