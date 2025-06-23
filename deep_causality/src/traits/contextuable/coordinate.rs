// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

/// Provides a generalized interface for N-dimensional coordinate access.
///
/// This trait is agnostic to geometry and is designed to support
/// both standard (Cartesian) coordinates and abstract representations such as:
/// - Curved spacetime manifolds
/// - Quaternionic rotations
/// - Symbolic embeddings (e.g., logical coordinates)
///
/// The trait provides only **index-based access** and leaves axis naming,
/// scaling, or metric behavior to higher-level abstractions.
///
/// # Example
/// ```
/// use deep_causality::prelude::Coordinate;
///
/// struct Vec3D {
///     x: f64,
///     y: f64,
///     z: f64,
/// }
///
/// impl Coordinate<f64> for Vec3D {
///     fn dimension(&self) -> usize {
///         3
///     }
///
///     fn coordinate(&self, index: usize) -> &f64 {
///         match index {
///             0 => &self.x,
///             1 => &self.y,
///             2 => &self.z,
///             _ => panic!("Index {} out of bounds for Vec3D", index),
///         }
///     }
/// }
/// ```
pub trait Coordinate<V> {
    /// Returns the number of dimensions defined in this coordinate system.
    fn dimension(&self) -> usize;

    /// Returns a reference to the value at a given axis index (0-based).
    ///
    /// # Panics
    /// May panic if index is out of bounds; implementations may handle this gracefully.
    fn coordinate(&self, index: usize) -> &V;
}
