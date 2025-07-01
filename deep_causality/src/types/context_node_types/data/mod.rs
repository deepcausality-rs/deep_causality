/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::hash::Hash;

use deep_causality_macros::Constructor;

use crate::prelude::Datable;

pub mod adjustable;
mod display;
pub mod identifiable;

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
///   type, satisfying the `Default + Copy + Clone + Hash + Eq + PartialEq` bounds.
///
#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data<T>
where
    T: Default + Copy + Clone + Hash + Eq + PartialEq,
{
    id: u64,
    data: T,
}

/// Implements the `Datable` trait for `Data<T>`.
///
/// This allows `Data<T>` to be used in contexts where a generic data container
/// is expected, providing methods to get and set the inner data payload.
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
