/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// should prevent circular dependencies to / from prelude
use crate::alias::alias_primitives::FloatType;
use crate::types::causal_types::causaloid::Causaloid;
use crate::types::causal_types::causaloid_graph::CausaloidGraph;
use crate::types::context_node_types::data::Data;
use crate::types::context_node_types::space::euclidean_space::EuclideanSpace;
use crate::types::context_node_types::space_time::euclidean_spacetime::EuclideanSpacetime;
use crate::types::context_node_types::symbol::base_symbol::BaseSymbol;
use crate::types::context_node_types::time::euclidean_time::EuclideanTime;
use crate::types::context_types::context_graph::Context;
use crate::types::context_types::contextoid::Contextoid;
use crate::types::model_types::model::Model;

use crate::{IntoEffectValue, NumericalValue, TeloidStore};
use std::collections::HashMap;

/// A type alias for the default `Model` configuration.
///
/// This alias represents a `Model` that operates with a standard set of generic
/// parameters, making it suitable for common causal modeling scenarios that
/// operate within a Euclidean and numerical framework.
///
/// Specifically, `BaseModel` is a `Model` parameterized as follows:
///
/// - **`Data<NumericalValue>`**: Used for its data component. `NumberType` is a
///   generic numeric type, typically an alias for a floating-point or integer,
///   allowing for flexible data representation within the model.
/// - **`EuclideanSpace`**: Defines the spatial context. This implies that
///   spatial relationships within this model adhere to standard 3D Euclidean geometry.
/// - **`EuclideanTime`**: Specifies the temporal context, utilizing a
///   Euclidean representation of time. This typically refers to a continuous,
///   linear progression of time.
/// - **`EuclideanSpacetime`**: Combines the Euclidean spatial and temporal
///   contexts into a unified spacetime representation, where both space and
///   time are treated with Euclidean properties.
/// - **`BaseSymbol`**: Provides a basic symbolic representation for elements
///   within the model, useful for labeling, identification, or abstract reasoning.
/// - **`FloatType` (x2)**: Two `FloatType` parameters, typically used for
///   internal calculations, scalar values, metrics, or other generic numerical
///   requirements within the `Model` structure, such as probabilities, weights,
///   or magnitudes.
///
/// This `BaseModel` is intended for general-purpose use cases where a standard
/// Euclidean and numerical context is sufficient, offering a consistent and
/// easily recognizable model structure for common causal reasoning and
/// simulation scenarios.
pub type BaseModel = Model<
    bool,
    bool,
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

/// A type alias for a default, general-purpose `Causaloid` configuration.
///
/// This alias represents a `Causaloid`—a single, identity-bearing causal unit—
/// configured with a standard set of generic parameters. It is designed for
/// common causal modeling scenarios that operate within a Euclidean and numerical
/// framework, providing a convenient and readable shorthand.
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
/// - **`EuclideanTime`**: Specifies the temporal context, using a Euclidean
///   representation of time. This typically refers to a continuous, linear progression
///   of time.
/// - **`EuclideanSpacetime`**: Combines the Euclidean spatial and temporal contexts
///   into a unified spacetime representation, where both space and time are treated
///   with Euclidean properties.
/// - **`BaseSymbol`**: Provides a basic symbolic representation for the causaloid,
///   useful for labeling, identification, or abstract reasoning.
/// - **`FloatType` (x2)**: Two `FloatType` parameters, typically used for internal
///   calculations, scalar values, or other generic numeric requirements within
///   the `Causaloid` structure, such as probabilities, weights, or magnitudes.
///
/// This `BaseCausaloid` is the standard choice for creating individual causal nodes
/// that are compatible with other "base" types like `BaseCausalGraph` and `BaseContext`,
/// ensuring a consistent and easily understandable modeling environment.
#[allow(type_alias_bounds)]
pub type BaseCausaloid<I: IntoEffectValue, O: IntoEffectValue> = Causaloid<
    I,
    O,
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

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
/// - **`EuclideanTime`**: Specifies the temporal context, using a Euclidean
///   representation of time. This typically refers to a continuous, linear progression
///   of time.
/// - **`EuclideanSpacetime`**: Combines the Euclidean spatial and temporal contexts
///   into a unified spacetime representation, where both space and time are treated
///   with Euclidean properties.
/// - **`BaseSymbol`**: Provides a basic symbolic representation for the causaloids,
///   useful for labeling, identification, or abstract reasoning.
/// - **`FloatType` (x2)**: Two `FloatType` parameters, typically used for internal
///   calculations, scalar values, or other generic numeric requirements within
///   the `Causaloid` structure, such as probabilities, weights, or magnitudes.
///
/// This `BaseCausaloidVec` is suitable for general-purpose use cases where a standard
/// Euclidean and numerical context is sufficient for defining and managing ordered
/// collections of causal entities. It offers a consistent and easily recognizable
/// way to organize causaloids for common causal modeling scenarios, such as
/// representing a sequence of events or a set of related causal agents.
#[allow(type_alias_bounds)]
pub type BaseCausaloidVec<I: IntoEffectValue, O: IntoEffectValue> = Vec<
    Causaloid<
        I,
        O,
        Data<NumericalValue>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >,
>;

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
/// - **`EuclideanTime`**: Specifies the temporal context, using a Euclidean
///   representation of time.
/// - **`EuclideanSpacetime`**: Combines the Euclidean spatial and temporal contexts
///   into a unified spacetime representation.
/// - **`BaseSymbol`**: Provides a basic symbolic representation for the causaloids.
/// - **`FloatType` (x2)**: Two `FloatType` parameters, typically used for internal
///   calculations, scalar values, or other generic numeric requirements within
///   the `Causaloid` structure.
///
/// This `BaseCausalMap` is suitable for general-purpose use cases where a standard
/// Euclidean and numerical context is sufficient for defining and managing causal
/// entities within a map structure. It offers a consistent and easily recognizable
/// way to organize causaloids for common causal modeling scenarios.
pub type BaseCausalMap = HashMap<
    usize,
    Causaloid<
        bool,
        bool,
        Data<NumericalValue>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >,
>;

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
/// - **`EuclideanTime`**: Specifies the temporal context, using a Euclidean
///   representation of time.
/// - **`EuclideanSpacetime`**: Combines the Euclidean spatial and temporal
///   contexts into a unified spacetime representation.
/// - **`BaseSymbol`**: Provides a basic symbolic representation for the causaloids.
/// - **`FloatType` (x2)**: Two `FloatType` parameters, typically used for internal
///   calculations, scalar values, or other generic numeric requirements within
///   the `Causaloid` structure.
///
/// This `BaseCausalGraph` is designed for general-purpose use cases where a
/// standard Euclidean and numerical context is sufficient, offering a consistent
/// and easily recognizable graph structure for common causal modeling scenarios.
pub type BaseCausalGraph = CausaloidGraph<
    Causaloid<
        bool,
        bool,
        Data<NumericalValue>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >,
>;

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
/// - **`BaseSymbol`**: Provides a basic symbolic representation for elements
///   within the context, useful for labeling, identification, or abstract
///   reasoning.
/// - **`FloatType` (x2)**: Two `FloatType` parameters, which are typically
///   used for internal calculations, scalar values, metrics, or other generic
///   numerical requirements within the `Context` structure, such as probabilities,
///   weights, or magnitudes.
///
/// This `BaseContext` is designed to be a sensible default for many applications,
/// offering a consistent and easily recognizable context structure for
/// general-purpose causal reasoning and data representation.
pub type BaseContext = Context<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

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
/// - **`BaseSymbol`**: Provides a basic symbolic representation for elements
///   within the context (a `Symboid`).
///
/// The two `FloatType` parameters correspond to the generic `VS` and `VT` types
/// required by the underlying `Contextoid` structure, representing the value types
/// for spatial and temporal coordinates, respectively.
///
/// This `BaseContextoid` is the standard choice for creating individual context nodes
/// that are compatible with other "base" types like `BaseContext` and `BaseCausalGraph`,
/// ensuring a consistent and easily understandable modeling environment.
pub type BaseContextoid = Contextoid<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

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
