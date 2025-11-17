/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    BaseCausaloid, BaseSymbol, Causaloid, CausaloidGraph, Data, EffectValue, EuclideanSpace,
    EuclideanSpacetime, EuclideanTime, FloatType, NumericalValue,
};

pub type RRMCausaloid = BaseCausaloid<EffectValue, EffectValue>;

pub type RCMCausalGraph = CausaloidGraph<
    Causaloid<
        // The input type of the causaloid.
        EffectValue,
        // The output type of the causaloid
        EffectValue,
        // Context type parameters. Unused in this example and thus set to some defaults.
        Data<NumericalValue>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >,
>;
