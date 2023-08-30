// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::{Constructor, Getters};

use crate::prelude::Datable;

mod display;
pub mod identifiable;

#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Dataoid<T>
where
    T: Copy + Default,
{
    #[getter(name = data_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    data: T,
}

// Type tag required for context.
impl<T> Datable for Dataoid<T> where T: Copy + Default {}
