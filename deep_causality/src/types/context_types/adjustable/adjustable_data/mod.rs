// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::Datable;

mod adjustable;
mod display;
mod identifiable;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableData<T>
    where T: Copy + Default,
{
    id: u64,
    data: T,
}

impl<T> AdjustableData<T>
    where T: Copy + Default,
{
    pub fn new(id: u64, data: T) -> Self
    {
        Self { id, data }
    }
}

impl<T> AdjustableData<T>
    where T: Copy + Default,
{
    pub fn data(&self) -> T {
        self.data
    }
}

// Type tag required for context.
impl<T> Datable for AdjustableData<T> where T: Copy + Default {}
