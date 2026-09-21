/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Ready-made `Context` and `Contextoid` configurations.
//!
//! `Base*` fixes concrete Euclidean space, time and spacetime; `Uniform*` uses the `Kind` enums so
//! one type signature serves every space, time and spacetime variant.

use deep_causality_core::{NumberType, NumericalValue};

/// The identifier of a `Context`.
///
/// The width is this crate's decision: it bounds how many contexts one program distinguishes,
/// and widening or narrowing it is a change to this crate rather than to the core vocabulary.
pub type ContextId = u64;

/// The identifier of a `Contextoid` within a `Context`.
///
/// The width is this crate's decision: it bounds how many contextoids one context holds, which
/// is the larger of the two counts and the one that moves first on an embedded target.
pub type ContextoidId = u64;

use crate::{
    Context, Contextoid, Data, EuclideanSpace, EuclideanSpacetime, EuclideanTime, SpaceKind,
    SpaceTimeKind, TimeKind,
};

/// A type alias for a default, general-purpose `Context` configuration.
///
/// This `BaseContext` alias represents a `Context` instance specifically configured
/// with a standard set of generic parameters, making it suitable for common
/// causal modeling scenarios that operate within a Euclidean and numerical framework.
///
/// It provides a convenient and readable shorthand for defining a `Context`
/// that encapsulates:
///
/// - **`Data<NumericalValue>`**: For handling general numerical data. `NumberType`
///   is typically an alias for a floating-point or integer type, allowing for
///   flexible data representation within the context.
/// - **`EuclideanSpace`**: Defines the spatial context using a standard
///   Euclidean coordinate system. This implies that spatial relationships
///   within this context adhere to Euclidean geometry.
/// - **`EuclideanTime`**: Specifies the temporal context, utilizing a
///   Euclidean representation of time. This typically refers to a continuous,
///   linear progression of time.
/// - **`EuclideanSpacetime`**: Combines the Euclidean spatial and temporal
///   contexts into a unified spacetime representation, where both space and
///   time are treated with Euclidean properties.
///
/// This `BaseContext` is designed to be a sensible default for many applications,
/// offering a consistent and easily recognizable context structure for
/// general-purpose causal reasoning and data representation.
pub type BaseContext =
    Context<Data<NumericalValue>, EuclideanSpace, EuclideanTime, EuclideanSpacetime>;

/// A type alias for a default, general-purpose `Contextoid` configuration.
///
/// This `BaseContextoid` alias represents a `Contextoid` instance—a single,
/// identity-bearing unit of context—configured with a standard set of generic
/// parameters. It is designed for common causal modeling scenarios that operate
/// within a Euclidean and numerical framework.
///
/// It provides a convenient and readable shorthand for defining a `Contextoid`
/// that encapsulates one of the following contextual roles:
///
/// - **`Data<NumericalValue>`**: For handling general numerical data (a `Datoid`). `NumberType`
///   is typically an alias for a floating-point or integer type.
/// - **`EuclideanSpace`**: Defines a spatial context using a standard
///   Euclidean coordinate system (a `Spaceoid`).
/// - **`EuclideanTime`**: Specifies a temporal context, utilizing a
///   Euclidean representation of time (a `Tempoid`).
/// - **`EuclideanSpacetime`**: Combines the Euclidean spatial and temporal
///   contexts into a unified spacetime representation (a `SpaceTempoid`).
///
///
/// This `BaseContextoid` is the standard choice for creating individual context nodes
/// that are compatible with other "base" types like `BaseContext` and `BaseCausalGraph`,
/// ensuring a consistent and easily understandable modeling environment.
pub type BaseContextoid =
    Contextoid<Data<NumericalValue>, EuclideanSpace, EuclideanTime, EuclideanSpacetime>;

/// A type alias for a default, general-purpose `Context` configuration that uses
/// abstract "kind" enums for its spatial, temporal, and symbolic contexts.
///
/// This `UniformContext` alias represents a `Context` instance configured with a
/// standard set of generic parameters, making it suitable for common causal
/// modeling scenarios where the specific underlying concrete types for space,
/// time, and symbols can vary but are represented by their respective "kind" enums.
///
/// It provides a convenient and readable shorthand for defining a `Context`
/// that encapsulates:
///
/// - **`Data<NumberType>`**: Used for its data component. `NumberType` is a
///   generic numeric type, typically an alias for a floating-point or integer,
///   allowing for flexible data representation within the context.
/// - **`SpaceKind`**: Defines the spatial context using an abstract `SpaceKind`
///   enum. This allows the context to operate with various spatial representations
///   (e.g., `EuclideanSpace`, `EcefSpace`, `NedSpace`, `GeoSpace`) without
///   changing the `Context`'s type signature, providing uniformity across different
///   spatial contexts.
/// - **`TimeKind`**: Specifies the temporal context using an abstract `TimeKind`
///   enum. This enables the context to handle different temporal representations
///   (e.g., `EuclideanTime`, `DiscreteTime`, `EntropicTime`, `LorentzianTime`)
///   flexibly, offering a uniform temporal interface.
/// - **`SpaceTimeKind`**: Combines the spatial and temporal contexts into a
///   unified spacetime representation using an abstract `SpaceTimeKind` enum,
///   allowing for various spacetime geometries (e.g., `EuclideanSpacetime`,
///   `LorentzianSpacetime`) in a uniform manner.
///
/// This `UniformContext` is designed to be a sensible default for many applications
/// requiring a flexible yet consistent context structure that can adapt to different
/// underlying spatial, temporal, and symbolic representations through their
/// respective `Kind` enums. It promotes code reusability and simplifies type
/// declarations when the exact concrete type of a context component is not
/// fixed but rather belongs to a set of predefined "kinds".
pub type UniformContext = Context<Data<NumberType>, SpaceKind, TimeKind, SpaceTimeKind>;

/// A type alias for a default, general-purpose `Contextoid` configuration that uses
/// abstract "kind" enums for its spatial, temporal, and symbolic contexts.
///
/// This `UniformContextoid` alias represents a `Contextoid` instance configured with a
/// standard set of generic parameters, making it suitable for common causal
/// modeling scenarios where the specific underlying concrete types for space,
/// time, and symbols can vary but are represented by their respective "kind" enums.
///
/// It provides a convenient and readable shorthand for defining a `Contextoid`
/// that encapsulates:
///
/// - **`Data<NumberType>`**: Used for its data component. `NumberType` is a
///   generic numeric type, typically an alias for a floating-point or integer,
///   allowing for flexible data representation within the contextoid.
/// - **`SpaceKind`**: Defines the spatial context using an abstract `SpaceKind`
///   enum. This allows the contextoid to operate with various spatial representations
///   (e.g., `EuclideanSpace`, `EcefSpace`, `NedSpace`, `GeoSpace`) without
///   changing the `Contextoid`'s type signature, providing uniformity across different
///   spatial contexts.
/// - **`TimeKind`**: Specifies the temporal context using an abstract `TimeKind`
///   enum. This enables the contextoid to handle different temporal representations
///   (e.g., `EuclideanTime`, `DiscreteTime`, `EntropicTime`, `LorentzianTime`)
///   flexibly, offering a uniform temporal interface.
/// - **`SpaceTimeKind`**: Combines the spatial and temporal contexts into a
///   unified spacetime representation using an abstract `SpaceTimeKind` enum,
///   allowing for various spacetime geometries (e.g., `EuclideanSpacetime`,
///   `LorentzianSpacetime`) in a uniform manner.
///
/// This `UniformContextoid` is designed to be a sensible default for many applications
/// requiring a flexible yet consistent contextoid structure that can adapt to different
/// underlying spatial, temporal, and symbolic representations through their
/// respective `Kind` enums. It promotes code reusability and simplifies type
/// declarations when the exact concrete type of a context component is not
/// fixed but rather belongs to a set of predefined "kinds".
pub type UniformContextoid = Contextoid<Data<NumberType>, SpaceKind, TimeKind, SpaceTimeKind>;
