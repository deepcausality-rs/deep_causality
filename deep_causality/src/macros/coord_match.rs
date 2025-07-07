/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

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
/// # Example
///
/// ```rust
/// use deep_causality::coord_match;
/// use deep_causality::*;
/// use deep_causality::errors::IndexError;
///
/// struct Vec3 {
///     x: f64,
///     y: f64,
///     z: f64,
/// }
///
/// impl Coordinate<f64> for Vec3 {
///     fn dimension(&self) -> usize {
///         3
///     }
///
///     fn coordinate(&self, index: usize) -> Result<&f64, IndexError> {
///         coord_match!(index,
///             0 => &self.x,
///             1 => &self.y,
///             2 => &self.z,
///         )
///     }
/// }
/// ```
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
            _ => Err($crate::errors::IndexError(format!(
                "Coordinate index out of bounds: {}", $index
            ))),
        }
    }};
}
