//! This module defines the `PropagatingValue` trait.

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::any::Any;
use std::fmt::Debug;

/// A trait for values that can be propagated through the causal graph.
/// It ensures that any type used in a causal function can be dynamically typed,
/// cloned, and compared for equality.
pub trait PropagatingValue: Debug + Send + Sync + 'static {
    /// Returns the value as a `dyn Any` reference.
    fn as_any(&self) -> &dyn Any;

    /// Creates a boxed clone of the value.
    fn clone_box(&self) -> Box<dyn PropagatingValue>;

    /// Performs a dynamic equality check.
    fn dyn_eq(&self, other: &dyn PropagatingValue) -> bool;
}

impl<T> PropagatingValue for T
where
    T: Debug + Clone + Send + Sync + PartialEq + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn PropagatingValue> {
        Box::new(self.clone())
    }

    fn dyn_eq(&self, other: &dyn PropagatingValue) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            self == other
        } else {
            false
        }
    }
}

impl Clone for Box<dyn PropagatingValue> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for dyn PropagatingValue {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}
