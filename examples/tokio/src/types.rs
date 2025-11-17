/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    BaseSymbol, Data, EuclideanSpace, EuclideanSpacetime, EuclideanTime, FloatType, Model,
    NumericalValue,
};

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
