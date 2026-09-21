/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
pub mod adjustable;
mod datable;
mod display;
mod identifiable;

/// A generic container for a piece of data, associated with a unique identifier.
///
/// `Data<T>` is a fundamental building block for representing discrete pieces of
/// information within a larger system, such as a context or a causal graph. It
/// wraps a data payload of type `T` and pairs it with a `u64` ID, allowing the
/// data to be uniquely identified and referenced.
///
/// The struct is designed to be lightweight and efficient, deriving `Copy` and `Clone`
/// to allow for easy duplication. The trait bounds on `T` ensure that the contained
/// data is also simple and value-like.
///
/// # Type Parameters
///
/// * `T`: The type of the data payload. It must be a simple, copyable, and comparable
///   type, satisfying the `Default + Clone + PartialEq` bounds. `Copy` is NOT required: it is
///   asked for only by the [`Adjustable`](crate::Adjustable) impl, because `ArrayGrid` is backed
///   by fixed-size arrays. A `Data<Vec<f64>>` is therefore a valid context node.
///
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Data<T>
where
    T: Default + Clone + PartialEq,
{
    id: ContextoidId,
    data: T,
}

impl<T> Data<T>
where
    T: Default + Clone + PartialEq,
{
    pub fn new(id: ContextoidId, data: T) -> Self {
        Self { id, data }
    }
}
