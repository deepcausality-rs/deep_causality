/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Ready-made `Context` and `Contextoid` configurations.
//!
//! `Base*` fixes the classical frame: Euclidean space, Newtonian time and Newtonian spacetime.
//! `Uniform*` uses the `Kind` enums so one type signature serves every space, time and spacetime
//! variant.

use deep_causality_core::{IdentificationValue, NumberType, NumericalValue};

/// The floating-point type this crate's ready-made aliases are built at.
pub type FloatType = f64;

/// The identifier of a `Context`.
///
/// Named here so a signature says which kind of identifier it takes, but the width is not this
/// crate's to choose: [`Identifiable::id`](deep_causality_core::Identifiable::id) returns
/// [`IdentificationValue`], and every context type implements it, so the two must agree. Widening
/// an identifier is a change to `deep_causality_core`, and this alias is written in terms of that
/// one so the coupling is visible rather than discovered.
pub type ContextId = IdentificationValue;

/// The identifier of a `Contextoid` within a `Context`.
///
/// Named for the role, with the width taken from [`IdentificationValue`] for the same reason as
/// [`ContextId`]: a contextoid's `id()` is its `Identifiable` impl, so it cannot differ.
pub type ContextoidId = IdentificationValue;

use crate::{
    Context, Contextoid, Data, EuclideanSpace, NewtonianSpacetime, NewtonianTime, SpaceKind,
    SpaceTimeKind, SubstrateRef, TimeKind,
};

/// A context in the classical frame: numeric data, positions in Euclidean space, absolute time, and
/// events of Newtonian spacetime.
///
/// - **`Data<NumericalValue>`**: numeric data nodes.
/// - **`EuclideanSpace`**: positions in three-dimensional Euclidean space.
/// - **`NewtonianTime`**: instants of absolute time.
/// - **`NewtonianSpacetime`**: events of Newtonian spacetime, in coordinates at rest in its
///   absolute space.
pub type BaseContext = Context<
    Data<NumericalValue>,
    EuclideanSpace<FloatType>,
    NewtonianTime<FloatType>,
    NewtonianSpacetime<FloatType>,
>;

/// A contextoid of [`BaseContext`]: one node holding numeric data (a `Datoid`), a position in
/// Euclidean space (a `Spaceoid`), an instant of absolute time (a `Tempoid`) or an event of
/// Newtonian spacetime (a `SpaceTempoid`).
pub type BaseContextoid = Contextoid<
    Data<NumericalValue>,
    EuclideanSpace<FloatType>,
    NewtonianTime<FloatType>,
    NewtonianSpacetime<FloatType>,
>;

/// A type alias for a default, general-purpose `Context` configuration that uses
/// abstract "kind" enums for its spatial, temporal and spacetime slots.
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
///   (e.g., `NewtonianTime`, `DiscreteTime`, `EntropicTime`, `MinkowskiTime`)
///   flexibly, offering a uniform temporal interface.
/// - **`SpaceTimeKind`**: Combines the spatial and temporal contexts into a
///   unified spacetime representation using an abstract `SpaceTimeKind` enum,
///   allowing for various spacetime geometries (e.g., `GalileanSpacetime`, `NewtonianSpacetime`,
///   `MinkowskiSpacetime`, `TangentSpacetime`) in a uniform manner.
///
/// This `UniformContext` is designed to be a sensible default for many applications
/// requiring a flexible yet consistent context structure that can adapt to different
/// underlying spatial, temporal and spacetime representations through their
/// respective `Kind` enums. It promotes code reusability and simplifies type
/// declarations when the exact concrete type of a context component is not
/// fixed but rather belongs to a set of predefined "kinds".
pub type UniformContext =
    Context<Data<NumberType>, SpaceKind<FloatType>, TimeKind<FloatType>, SpaceTimeKind<FloatType>>;

/// A type alias for a default, general-purpose `Contextoid` configuration that uses
/// abstract "kind" enums for its spatial, temporal and spacetime slots.
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
///   (e.g., `NewtonianTime`, `DiscreteTime`, `EntropicTime`, `MinkowskiTime`)
///   flexibly, offering a uniform temporal interface.
/// - **`SpaceTimeKind`**: Combines the spatial and temporal contexts into a
///   unified spacetime representation using an abstract `SpaceTimeKind` enum,
///   allowing for various spacetime geometries (e.g., `GalileanSpacetime`, `NewtonianSpacetime`,
///   `MinkowskiSpacetime`, `TangentSpacetime`) in a uniform manner.
///
/// This `UniformContextoid` is designed to be a sensible default for many applications
/// requiring a flexible yet consistent contextoid structure that can adapt to different
/// underlying spatial, temporal and spacetime representations through their
/// respective `Kind` enums. It promotes code reusability and simplifies type
/// declarations when the exact concrete type of a context component is not
/// fixed but rather belongs to a set of predefined "kinds".
pub type UniformContextoid = Contextoid<
    Data<NumberType>,
    SpaceKind<FloatType>,
    TimeKind<FloatType>,
    SpaceTimeKind<FloatType>,
>;

/// The shape a store that holds references accepts: the three `Kind` enums, for which the
/// projection is total, and a data node that holds where its value lives rather than the value.
pub type SubstrateContext = Context<
    Data<SubstrateRef>,
    SpaceKind<FloatType>,
    TimeKind<FloatType>,
    SpaceTimeKind<FloatType>,
>;

/// The contextoid of a [`SubstrateContext`].
pub type SubstrateContextoid = Contextoid<
    Data<SubstrateRef>,
    SpaceKind<FloatType>,
    TimeKind<FloatType>,
    SpaceTimeKind<FloatType>,
>;
