/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// The macros below are code generators used to implement type extensions with minimal boilerplate.
// See deep_causality/src/extensions

#[macro_export]
macro_rules! make_len {
    () => {
        fn len(&self) -> usize {
            self.len()
        }
    };
}

#[macro_export]
macro_rules! make_is_empty {
    () => {
        fn is_empty(&self) -> bool {
            self.is_empty()
        }
    };
}

#[macro_export]
macro_rules! make_get_all_items {
    () => {
        fn get_all_items(&self) -> Vec<&T> {
            let mut all: Vec<&T> = Vec::new();
            for item in self {
                all.push(&item)
            }
            all
        }
    };
}

#[macro_export]
macro_rules! make_get_all_map_items {
    () => {
        fn get_all_items(&self) -> Vec<&V> {
            self.values().collect::<Vec<&V>>()
        }
    };
}

#[macro_export]
macro_rules! make_array_to_vec {
    () => {
        fn to_vec(&self) -> Vec<T> {
            self.to_vec()
        }
    };
}

#[macro_export]
macro_rules! make_map_to_vec {
    () => {
        fn to_vec(&self) -> Vec<V> {
            self.values().cloned().collect()
        }
    };
}

#[macro_export]
macro_rules! make_vec_to_vec {
    () => {
        fn to_vec(&self) -> Vec<T> {
            self.clone()
        }
    };
}

#[macro_export]
macro_rules! make_vec_deq_to_vec {
    () => {
        fn to_vec(&self) -> Vec<T> {
            let mut v = Vec::with_capacity(self.len());
            let mut deque = self.clone(); // clone to avoid mutating the original

            for item in deque.make_contiguous().iter() {
                v.push(item.clone());
            }

            v
        }
    };
}

#[macro_export]
macro_rules! make_find_from_map_values {
    () => {
        fn get_item_by_id(&self, id: IdentificationValue) -> Option<&V> {
            self.values().find(|item| item.id() == id)
        }
    };
}

#[macro_export]
macro_rules! make_find_from_iter_values {
    () => {
        fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T> {
            self.iter().find(|item| item.id() == id)
        }
    };
}

/// A macro to concisely and safely implement coordinate indexing with bounds checking.
///
/// This macro expands to a `match` expression that returns `Ok(&value)` for a given
/// index or an `Err(IndexError)` if the index is out of bounds. It is intended for use
/// in implementations of the `Coordinate` trait where types expose fixed-dimension
/// spatial data (e.g., x/y/z, lat/lon/alt, quaternion components, etc.).
///
/// # Parameters
///
/// - `$index`: The index expression to match (usually from a function argument).
/// - Each `N => EXPR`: A mapping from a literal index (`usize`) to a reference expression
///   representing the coordinate value at that index.
///
/// # Return
///
/// Returns a `Result<&T, IndexError>`, where `T` is the type of the coordinate
/// component (e.g., `f64`). This allows the caller to gracefully handle invalid
/// index accesses without panicking.
///
/// # Features
///
/// - Enforces match completeness with a fallback `_ => Err(...)`.
/// - Accepts a trailing comma.
/// - Avoids boilerplate code duplication across coordinate types.
///
/// # Errors
///
/// Returns `Err(IndexError)` if the index is greater than or equal to the number of
/// defined coordinate components.
///
/// # See Also
///
/// - `Coordinate` trait
/// - `IndexError` type
///
/// # Note
///
/// This macro must be imported using `#[macro_use]` if you're not using the 2018+ Rust edition
/// module system with `pub use crate::macros::coord_match;`.
#[macro_export]
macro_rules! coord_match {
    ($index:expr, $( $i:literal => $val:expr ),+ $(,)?) => {{
        match $index {
            $(
                $i => Ok($val),
            )+
            _ => Err(IndexError(format!(
                "Coordinate index out of bounds: {}", $index
            ))),
        }
    }};
}
