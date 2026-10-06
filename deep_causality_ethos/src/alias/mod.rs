/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::TeloidStore;
use deep_causality::NumericalValue;
use deep_causality_context::{Data, EuclideanSpace, NewtonianSpacetime, NewtonianTime};

/// The floating-point type this crate's ready-made aliases are built at.
pub type FloatType = f64;

pub type TeloidTag = &'static str;
pub type TeloidID = u64;

/// A type alias for a default, general-purpose `TeloidStore` configuration.
///
/// This `BaseTeloidStore` alias represents a `TeloidStore` instance—a specialized
/// data structure for managing and querying teloids (temporal causal units)—
/// configured with a standard set of generic parameters. It is designed for
/// common causal modeling scenarios in the classical frame: Euclidean space, Newtonian time
/// and Newtonian spacetime, with numerical data.
///
/// It provides a convenient and readable shorthand for defining a `TeloidStore`
/// that encapsulates:
///
/// - **`Data<NumericalValue>`**: For handling general numerical data associated with teloids.
///   `NumberType` is typically an alias for a floating-point or integer type,
///   allowing for flexible data representation.
/// - **`EuclideanSpace`**: Defines the spatial context of the teloids using a standard
///   Euclidean coordinate system. This implies that spatial relationships
///   within this store adhere to Euclidean geometry.
/// - **[`NewtonianTime`]**: Specifies the temporal context as instants of absolute time, the
///   time of classical spacetime. The duration between two instants is the same for every
///   observer (Weatherall 2021, §3).
/// - **[`NewtonianSpacetime`]**: Combines space and time into events of Newtonian spacetime, in
///   coordinates at rest in absolute space. Its spatial metric is the degenerate
///   `diag(0, 1, 1, 1)` on `(t, x, y, z)`, so the distance between two events is their
///   distance in absolute space, defined at any two times
///   (Malament 2012, Prop. 4.1.2; Weatherall 2021, §4).
/// - **`FloatType`**: The scalar the three geometric node types are built at. The same
///   `FloatType` parameterises `EuclideanSpace`, `NewtonianTime`, and `NewtonianSpacetime`,
///   so every coordinate a norm reads out of this store is measured in one scalar type.
///
/// The citations are given in full on [`NewtonianTime`] and [`NewtonianSpacetime`].
///
/// This `BaseTeloidStore` is designed to be a sensible default for many applications,
/// offering a consistent and easily recognizable structure for managing and
/// querying temporal causal data in general-purpose causal reasoning and
/// simulation scenarios.
pub type BaseTeloidStore = TeloidStore<
    Data<NumericalValue>,
    EuclideanSpace<FloatType>,
    NewtonianTime<FloatType>,
    NewtonianSpacetime<FloatType>,
>;
