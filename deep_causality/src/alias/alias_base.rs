/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
/// The floating-point type this crate works in.
pub type FloatType = f64;

use crate::{Causaloid, CausaloidGraph, Model};
use deep_causality_context::BaseContext;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A type alias for the default `Model` configuration.
///
/// This alias represents a `Model` that operates with a standard set of generic
/// parameters, making it suitable for common causal modeling scenarios in
/// the classical frame of `BaseContext`: Euclidean space, Newtonian time and Newtonian
/// spacetime, with numerical data.
///
/// Specifically, `BaseModel` is a `Model` parameterized as follows:
///
/// - **`Data<NumericalValue>`**: Used for its data component. `NumberType` is a
///   generic numeric type, typically an alias for a floating-point or integer,
///   allowing for flexible data representation within the model.
/// - **`EuclideanSpace`**: Defines the spatial context. This implies that
///   spatial relationships within this model adhere to standard 3D Euclidean geometry.
/// - **[`NewtonianTime`](deep_causality_context::NewtonianTime)**: Specifies the temporal
///   context as instants of absolute time, the time of classical spacetime. The duration between
///   two instants is the same for every observer (Weatherall 2021, §3).
/// - **[`NewtonianSpacetime`](deep_causality_context::NewtonianSpacetime)**: Combines space and
///   time into events of Newtonian spacetime, in coordinates at rest in absolute space. Its
///   spatial metric is the degenerate `diag(0, 1, 1, 1)` on `(t, x, y, z)`, so the distance
///   between two events is their distance in absolute space, defined at any two times
///   (Malament 2012, Prop. 4.1.2; Weatherall 2021, §4).
/// - **`FloatType` (x2)**: Two `FloatType` parameters, typically used for
///   internal calculations, scalar values, metrics, or other generic numerical
///   requirements within the `Model` structure, such as probabilities, weights,
///   or magnitudes.
///
/// The citations are given in full on [`NewtonianTime`](deep_causality_context::NewtonianTime)
/// and [`NewtonianSpacetime`](deep_causality_context::NewtonianSpacetime).
///
/// This `BaseModel` is intended for general-purpose use cases where the classical context of
/// `BaseContext` with numerical data is sufficient, offering a consistent and easily
/// recognizable model structure for common causal reasoning and simulation scenarios.
pub type BaseModel = Model<bool, bool, BaseContext>;

/// A type alias for a default, general-purpose `Causaloid` configuration.
///
/// This alias represents a `Causaloid`—a single, identity-bearing causal unit—
/// configured with a standard set of generic parameters. It is designed for
/// common causal modeling scenarios in the classical frame of `BaseContext` (Euclidean space,
/// Newtonian time and Newtonian spacetime) with numerical data, providing a convenient and
/// readable shorthand.
///
/// Each `BaseCausaloid` is parameterized with the following concrete types,
/// defining its default context and data handling:
///
/// - **`Data<NumericalValue>`**: Represents the data component associated with the causaloid.
///   `NumberType` is a generic numeric type, typically a floating-point or integer,
///   allowing for flexible data representation.
/// - **`EuclideanSpace`**: Defines the spatial context of the causaloid within a
///   standard 3D Euclidean coordinate system. This implies that spatial relationships
///   are governed by Euclidean geometry.
/// - **[`NewtonianTime`](deep_causality_context::NewtonianTime)**: Specifies the temporal
///   context as instants of absolute time, the time of classical spacetime. The duration between
///   two instants is the same for every observer (Weatherall 2021, §3).
/// - **[`NewtonianSpacetime`](deep_causality_context::NewtonianSpacetime)**: Combines space and
///   time into events of Newtonian spacetime, in coordinates at rest in absolute space. Its
///   spatial metric is the degenerate `diag(0, 1, 1, 1)` on `(t, x, y, z)`, so the distance
///   between two events is their distance in absolute space, defined at any two times
///   (Malament 2012, Prop. 4.1.2; Weatherall 2021, §4).
/// - **`FloatType` (x2)**: Two `FloatType` parameters, typically used for internal
///   calculations, scalar values, or other generic numeric requirements within
///   the `Causaloid` structure, such as probabilities, weights, or magnitudes.
///
/// The citations are given in full on [`NewtonianTime`](deep_causality_context::NewtonianTime)
/// and [`NewtonianSpacetime`](deep_causality_context::NewtonianSpacetime).
///
/// This `BaseCausaloid` is the standard choice for creating individual causal nodes
/// that are compatible with other "base" types like `BaseCausalGraph` and `BaseContext`,
/// ensuring a consistent and easily understandable modeling environment.
#[allow(type_alias_bounds)]
pub type BaseCausaloid<I, O> = Causaloid<I, O, (), Arc<RwLock<BaseContext>>>;

/// A type alias for a `Vec` (vector) containing `BaseCausaloid` instances.
///
/// This alias provides a convenient shorthand for a collection of causaloids,
/// where each causaloid adheres to a standard "base" configuration. It's designed
/// to represent an ordered list of `Causaloid` instances that share a common
/// set of generic parameters, making it suitable for scenarios where multiple
/// causaloids need to be grouped or processed together.
///
/// Each `Causaloid` within this vector is parameterized with the following
/// concrete types, defining its default context and data handling:
///
/// - **`Data<NumericalValue>`**: Represents the data component associated with each causaloid.
///   `NumberType` is a generic numeric type, typically a floating-point or integer,
///   allowing for flexible data representation.
/// - **`EuclideanSpace`**: Defines the spatial context of the causaloids within a
///   standard 3D Euclidean coordinate system. This implies that spatial relationships
///   are governed by Euclidean geometry.
/// - **[`NewtonianTime`](deep_causality_context::NewtonianTime)**: Specifies the temporal
///   context as instants of absolute time, the time of classical spacetime. The duration between
///   two instants is the same for every observer (Weatherall 2021, §3).
/// - **[`NewtonianSpacetime`](deep_causality_context::NewtonianSpacetime)**: Combines space and
///   time into events of Newtonian spacetime, in coordinates at rest in absolute space. Its
///   spatial metric is the degenerate `diag(0, 1, 1, 1)` on `(t, x, y, z)`, so the distance
///   between two events is their distance in absolute space, defined at any two times
///   (Malament 2012, Prop. 4.1.2; Weatherall 2021, §4).
/// - **`FloatType` (x2)**: Two `FloatType` parameters, typically used for internal
///   calculations, scalar values, or other generic numeric requirements within
///   the `Causaloid` structure, such as probabilities, weights, or magnitudes.
///
/// The citations are given in full on [`NewtonianTime`](deep_causality_context::NewtonianTime)
/// and [`NewtonianSpacetime`](deep_causality_context::NewtonianSpacetime).
///
/// This `BaseCausaloidVec` is suitable for general-purpose use cases where the classical
/// context of `BaseContext` with numerical data is sufficient for defining and managing ordered
/// collections of causal entities. It offers a consistent and easily recognizable
/// way to organize causaloids for common causal modeling scenarios, such as
/// representing a sequence of events or a set of related causal agents.
#[allow(type_alias_bounds)]
pub type BaseCausaloidVec<I, O> = Vec<Causaloid<I, O, (), Arc<RwLock<BaseContext>>>>;

/// A type alias for a `HashMap` that stores `BaseCausaloid` instances, typically indexed by their unique identifiers.
///
/// This alias provides a convenient shorthand for a collection of causaloids,
/// where each causaloid adheres to a standard "base" configuration. It's designed
/// to represent a mapping from an integer ID (e.g., a node index or a unique identifier)
/// to a `Causaloid` instance.
///
/// The `BaseCausaloid` type, which forms the value of this map, is parameterized
/// with the following concrete types:
///
/// - **`Data<NumericalValue>`**: Represents the data component associated with each causaloid.
///   `NumberType` is a generic numeric type, typically a floating-point or integer.
/// - **`EuclideanSpace`**: Defines the spatial context of the causaloids within a
///   standard Euclidean coordinate system.
/// - **[`NewtonianTime`](deep_causality_context::NewtonianTime)**: Specifies the temporal
///   context as instants of absolute time, the time of classical spacetime
///   (Weatherall 2021, §3).
/// - **[`NewtonianSpacetime`](deep_causality_context::NewtonianSpacetime)**: Combines space and
///   time into events of Newtonian spacetime. The distance between two events is their distance
///   in absolute space, at any two times (Malament 2012, Prop. 4.1.2; Weatherall 2021, §4).
/// - **`FloatType` (x2)**: Two `FloatType` parameters, typically used for internal
///   calculations, scalar values, or other generic numeric requirements within
///   the `Causaloid` structure.
///
/// The citations are given in full on [`NewtonianTime`](deep_causality_context::NewtonianTime)
/// and [`NewtonianSpacetime`](deep_causality_context::NewtonianSpacetime).
///
/// This `BaseCausalMap` is suitable for general-purpose use cases where the classical context
/// of `BaseContext` with numerical data is sufficient for defining and managing causal entities
/// within a map structure. It offers a consistent and easily recognizable
/// way to organize causaloids for common causal modeling scenarios.
pub type BaseCausalMap = HashMap<usize, Causaloid<bool, bool, (), Arc<RwLock<BaseContext>>>>;

pub type BenchmarkCausalMap = HashMap<usize, BaseCausaloid<f64, bool>>;

/// A type alias for a `CausaloidGraph` composed of `BaseCausaloid` instances.
///
/// This alias provides a convenient shorthand for defining a causal graph where
/// each node (causaloid) adheres to a standard "base" configuration.
///
/// Specifically, `BaseCausalGraph` is a `CausaloidGraph` parameterized by a `Causaloid`
/// that uses the following concrete types for its generic parameters:
/// - **`Data<NumericalValue>`**: Represents the data associated with each causaloid,
///   using a generic `NumberType` (typically a floating-point or integer type).
/// - **`EuclideanSpace`**: Defines the spatial context of the causaloids within
///   a standard Euclidean coordinate system.
/// - **[`NewtonianTime`](deep_causality_context::NewtonianTime)**: Specifies the temporal
///   context as instants of absolute time, the time of classical spacetime
///   (Weatherall 2021, §3).
/// - **[`NewtonianSpacetime`](deep_causality_context::NewtonianSpacetime)**: Combines space and
///   time into events of Newtonian spacetime. The distance between two events is their distance
///   in absolute space, at any two times (Malament 2012, Prop. 4.1.2; Weatherall 2021, §4).
/// - **`FloatType` (x2)**: Two `FloatType` parameters, typically used for internal
///   calculations, scalar values, or other generic numeric requirements within
///   the `Causaloid` structure.
///
/// The citations are given in full on [`NewtonianTime`](deep_causality_context::NewtonianTime)
/// and [`NewtonianSpacetime`](deep_causality_context::NewtonianSpacetime).
///
/// This `BaseCausalGraph` is designed for general-purpose use cases where the classical context
/// of `BaseContext` with numerical data is sufficient, offering a consistent and easily
/// recognizable graph structure for common causal modeling scenarios.
pub type BaseCausalGraph = CausaloidGraph<Causaloid<bool, bool, (), Arc<RwLock<BaseContext>>>>;
