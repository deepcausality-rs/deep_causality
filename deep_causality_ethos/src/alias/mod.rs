/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::TeloidStore;
use deep_causality::{
    BaseSymbol, Data, EuclideanSpace, EuclideanSpacetime, EuclideanTime, FloatType, NumericalValue,
};

pub type TeloidTag = &'static str;
pub type TeloidID = u64;

/// A type alias for a default, general-purpose `TeloidStore` configuration.
///
/// This `BaseTeloidStore` alias represents a `TeloidStore` instance—a specialized
/// data structure for managing and querying teloids (temporal causal units)—
/// configured with a standard set of generic parameters. It is designed for
/// common causal modeling scenarios that operate within a Euclidean and numerical
/// framework.
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
/// - **`EuclideanTime`**: Specifies the temporal context, utilizing a
///   Euclidean representation of time. This typically refers to a continuous,
///   linear progression of time.
/// - **`EuclideanSpacetime`**: Combines the Euclidean spatial and temporal
///   contexts into a unified spacetime representation, where both space and
///   time are treated with Euclidean properties.
/// - **`BaseSymbol`**: Provides a basic symbolic representation for teloids
///   within the store, useful for labeling, identification, or abstract
///   reasoning.
/// - **`FloatType` (x2)**: Two `FloatType` parameters, which are typically
///   used for internal calculations, scalar values, metrics, or other generic
///   numerical requirements within the `TeloidStore` structure, such as probabilities,
///   weights, or magnitudes.
///
/// This `BaseTeloidStore` is designed to be a sensible default for many applications,
/// offering a consistent and easily recognizable structure for managing and
/// querying temporal causal data in general-purpose causal reasoning and
/// simulation scenarios.
pub type BaseTeloidStore = TeloidStore<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;
