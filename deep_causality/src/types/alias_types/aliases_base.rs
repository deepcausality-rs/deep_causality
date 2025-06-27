/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::prelude::{
    BaseSymbol, Causaloid, CausaloidGraph, Context, Contextoid, Data, EuclideanSpace,
    EuclideanSpacetime, EuclideanTime, FloatType, Model, NumberType,
};
use std::collections::HashMap;

pub type BaseModel = Model<
    Data<NumberType>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

pub type BaseCausaloid = Causaloid<
    Data<NumberType>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

pub type BaseCausaloidVec = Vec<
    Causaloid<
        Data<NumberType>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >,
>;

pub type BaseCausalMap = HashMap<
    usize,
    Causaloid<
        Data<NumberType>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >,
>;

pub type BaseCausalGraph = CausaloidGraph<
    Causaloid<
        Data<NumberType>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >,
>;

// Default type alias for basic context. It's used in tests
pub type BaseContext = Context<
    Data<NumberType>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

pub type BaseContextoid = Contextoid<
    Data<NumberType>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;
