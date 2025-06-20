// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{ContextoidType, Identifiable, TimeScale};


/// Represents any entity that participates in a causal context graph.
///
/// This trait defines the unified interface over any entity that may be:
/// - A data node
/// - A spatial or temporal marker
/// - A symbolic atom
/// - A spacetime event
///
/// It is designed to **abstract over the underlying causal semantics**
/// while retaining compile-time type safety and minimal trait bounds.
///
/// # Type Parameters
/// - `D`: A [`Datable`] node (e.g., sensor reading, fact, entity)
/// - `S`: A [`Spatial`] node
/// - `T`: A [`Temporable`] node
/// - `ST`: A [`SpaceTemporal`] node (4D entity)
/// - `SYM`: A [`Symbolic`] node (logical/abstract)
/// - `V`: The numeric or symbolic coordinate type
///
/// # Design Note
/// This trait is the dispatch point for `ContextoidType`, allowing static or
/// dynamic graph traversal based on node kind. It intentionally generalizes
/// over all possible causal node roles.
pub trait Contextuable<D, S, T, ST, SYM, V>: Identifiable
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    /// Returns a reference to the type-erased node variant.
    ///
    /// Use this to determine the role of the current node (data, space, time, etc.)
    /// and then downcast or dispatch accordingly.
    fn vertex_type(&self) -> &ContextoidType<D, S, T, ST, SYM, V>;
}


/// Represents data-bearing entities in a causal context graph.
///
/// This trait marks nodes or values that carry domain-specific data
/// relevant to inference, observation, or explanation. It extends
/// [`Identifiable`] to ensure that each instance has a unique identity.
///
/// This trait is intentionally left minimal to allow full flexibility
/// in how data is modeled. You may wrap sensor input, encoded strings,
/// discrete values, or even external references.
///
/// # Example
/// ```
/// use deep_causality::prelude::{Datable, Identifiable};
///
/// struct SensorReading { id: u64, value: f64 }
/// impl Identifiable for SensorReading { fn id(&self) -> u64 { self.id } }
/// impl Datable for SensorReading {}
/// ```
pub trait Datable: Identifiable {}

/// Defines a measurable relationship between two entities within the same space.
///
/// This trait abstracts over geometric or abstract "distance" computations.
/// It may represent physical distance, influence strength, semantic similarity,
/// or any scalar that satisfies the symmetry and triangle inequality properties
/// of a metric space—though these are not enforced by the trait.
///
/// Use this trait for types where **numeric comparison** between entities
/// makes sense (e.g. Euclidean points, time series, weighted graphs).
///
/// # Notes
/// - Units must match across implementations of `V`
/// - If you're implementing this in a curved or non-Euclidean space,
///   you may need to inject a metric tensor
///
pub trait Metric<V> {
    /// Computes a scalar distance or influence between `self` and `other`.
    fn distance(&self, other: &Self) -> V;
}

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

/// Represents coordinate-bearing entities that also implement distance metrics.
///
/// This trait is a composition of [`Coordinate`] and [`Metric`], intended for
/// **Euclidean or pseudo-Euclidean spaces** where distance can be meaningfully
/// computed between coordinate positions.
///
/// # Example Use Cases
/// - 3D physical space (x, y, z)
/// - Latent vector spaces in embeddings
/// - Discrete grids with uniform spacing
pub trait MetricCoordinate<V>: Coordinate<V> + Metric<V> {}

/// Represents entities that have intrinsic temporal properties.
///
/// This trait provides access to both the **scale** (e.g. seconds, minutes)
/// and **unit value** (e.g. timestamp, frame number) associated with a temporal point.
///
/// Use this for any node, edge, or context that evolves over time or contributes
/// to time-dependent reasoning.
///
/// # Notes
/// The numeric type `V` must support ordering and arithmetic if used for inference.
pub trait Temporable<V>: Identifiable {
    /// Returns the unit scale of time (e.g. `TimeScale::Milliseconds`).
    fn time_scale(&self) -> TimeScale;

    /// Returns a reference to the numeric time unit (e.g. 0, 100, 32768).
    fn time_unit(&self) -> &V;
}

/// Marks entities that have spatial semantics.
///
/// This is a composite trait that combines:
/// - [`Identifiable`] — uniquely tracked nodes
/// - [`Coordinate<V>`] — N-dimensional position or location
///
/// It does **not** require a metric, allowing support for:
/// - Symbolic zones
/// - Discrete lattice structures
/// - Topological graphs with no distance definition
///
/// Use this to model anything *located* in space—regardless of how space is defined.
pub trait Spatial<V>: Identifiable + Coordinate<V> {}

/// Combines spatial and temporal semantics into a 4D spacetime model.
///
/// This is ideal for modeling causal entities that exist at a particular
/// **spatial location** and **point in time**. The `t()` method supplements
/// the coordinate system with a direct accessor for the temporal axis.
///
/// This trait enables compatibility with:
/// - Newtonian and Einsteinian physics
/// - Sensor frames
/// - 4D event graphs
///
/// # Note
/// The actual meaning of `t()` depends on the context—e.g., wall clock time,
/// simulation ticks, or a relativistic coordinate frame.
pub trait SpaceTemporal<V>: Identifiable + Spatial<V> + Temporable<V> {
    /// Returns the value associated with the temporal (4th) dimension.
    fn t(&self) -> &V;
}

/// Represents a symbolic, logical, or linguistic identity.
///
/// This trait allows integration of abstract knowledge representations
/// into a unified context system—such as:
/// - Atoms (`"A"`)
/// - Named terms (`goal(X)`)
/// - Logical constructs (`∀x.P(x) → Q(x)`)
///
/// The `Repr` type is intentionally generic to support structured representations:
/// enums, trees, strings, or AST nodes.
///
/// # Example
/// ```
/// use deep_causality::prelude::{Identifiable, Symbolic};
///
/// struct RuleNode { id: u64, term: String }
///
/// impl Identifiable for RuleNode {fn id(&self) -> u64 {
///         self.id
///     }}
///
/// impl Symbolic for RuleNode {
///     type Repr = String;
///     fn symbol(&self) -> &Self::Repr { &self.term }
/// }
/// ```
pub trait Symbolic: Identifiable {
    /// The representation type used to encode the symbolic value.
    type Repr;

    /// Returns a reference to the symbolic representation.
    fn symbol(&self) -> &Self::Repr;
}
