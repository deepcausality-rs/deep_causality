/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    BaseCausaloid, BaseSymbol, CSM, Data, EffectValue, EuclideanSpace, EuclideanSpacetime,
    EuclideanTime,
};

pub type CsmCausaloid = BaseCausaloid<EffectValue, bool>;

/// # Type Parameters
/// - `I`: The type of the input effect value, must implement `IntoEffectValue`.
/// - `O`: The type of the output effect value, must implement `IntoEffectValue`.
/// - `D`: The type for data context, must implement `Datable` and `Clone`.
/// - `S`: The type for spatial context, must implement `Spatial<VS>` and `Clone`.
/// - `T`: The type for temporal context, must implement `Temporal<VT>` and `Clone`.
/// - `ST`: The type for spatiotemporal context, must implement `SpaceTemporal<VS, VT>` and `Clone`.
/// - `SYM`: The type for symbolic context, must implement `Symbolic` and `Clone`.
/// - `VS`: The value type for spatial data, must implement `Clone`.
/// - `VT`: The value type for temporal data, must implement `Clone`.
pub type ServerCSM = CSM<
    EffectValue,
    bool,
    Data<f64>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    f64,
    f64,
>;
