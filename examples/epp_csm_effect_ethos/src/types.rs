/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    BaseCausaloid, BaseSymbol, Data, EffectEthos, EuclideanSpace, EuclideanSpacetime,
    EuclideanTime, FloatType, NumericalValue,
};

pub type CsmCausaloid = BaseCausaloid<NumericalValue, bool>;

pub type CsmEthos = EffectEthos<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;
