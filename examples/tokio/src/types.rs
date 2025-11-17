/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    BaseSymbol, Data, EuclideanSpace, EuclideanSpacetime, EuclideanTime, FloatType, Model,
    NumericalValue,
};

// # Model Type Parameters
// - `I`: The type of the input effect value, must implement `IntoEffectValue`.
// - `O`: The type of the output effect value, must implement `IntoEffectValue`.
// -- These are only relevant when using context.
// - `D`: The type for data context, must implement `Datable` and `Clone`.
// - `S`: The type for spatial context, must implement `Spatial<VS>` and `Clone`.
// - `T`: The type for temporal context, must implement `Temporal<VT>` and `Clone`.
// - `ST`: The type for spatiotemporal context, must implement `SpaceTemporal<VS, VT>` and `Clone`.
// - `SYM`: The type for symbolic context, must implement `Symbolic` and `Clone`.
// - `VS`: The value type for spatial data, must implement `Clone`.
// - `VT`: The value type for temporal data, must implement `Clone`.
pub type BaseModelTokio = Model<
    NumericalValue,
    bool,
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;
