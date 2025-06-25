// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::hash::Hash;

use deep_causality_macros::Constructor;

use crate::prelude::Datable;

pub mod adjustable;
mod display;
pub mod identifiable;

#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data<T>
where
    T: Default + Copy + Clone + Hash + Eq + PartialEq,
{
    id: u64,
    data: T,
}

// Type tag required for context.
impl<T> Datable for Data<T>
where
    T: Default + Copy + Clone + Hash + Eq + PartialEq,
{
    type Data = T;

    fn get_data(&self) -> Self::Data {
        self.data
    }

    fn set_data(&mut self, value: Self::Data) {
        self.data = value;
    }
}
