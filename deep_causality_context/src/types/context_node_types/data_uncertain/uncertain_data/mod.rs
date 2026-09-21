/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ContextoidId;
use deep_causality_uncertain::{RandScalar, Uncertain};

mod adjustable;
mod datable;
mod display;
mod identifiable;

/// A context node holding a real-valued uncertain quantity.
///
/// # The scalar is a parameter
///
/// `R` is whatever scalar the quantity is carried at. Making it one costs `Context` nothing:
/// [`Datable`](crate::Datable) has an associated `Data` type rather than a parameter of its own, so
/// `UncertainData<R>` is just another `D` in `Context<D, S, T, ST>` — exactly as the
/// plain [`Data<T>`](crate::Data) already is inside [`BaseContext`](crate::BaseContext).
#[derive(Debug, Clone)]
pub struct UncertainData<R: RandScalar> {
    id: ContextoidId,
    data: Uncertain<R>,
}

impl<R: RandScalar> UncertainData<R> {
    pub fn new(id: ContextoidId, data: Uncertain<R>) -> Self {
        Self { id, data }
    }
}
