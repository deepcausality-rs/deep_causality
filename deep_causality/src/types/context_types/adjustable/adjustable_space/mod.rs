// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::marker::PhantomData;

use deep_causality_macros::{Constructor, Getters};

mod identifiable;
mod adjustable;
mod display;

#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableSpace<T>
    where T: Copy + Default
{
    id: u64,
    x: i64,
    y: i64,
    z: i64,
    ty: PhantomData<T>, // Need to bind T
}
