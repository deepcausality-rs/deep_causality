/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// should prevent circular dependencies to / from prelude
use crate::types::alias_types::alias_primitives::{FloatType, NumberType};
use crate::types::causal_types::causaloid::Causaloid;
use crate::types::causal_types::causaloid_graph::CausaloidGraph;
use crate::types::context_types::context_graph::Context;
use crate::types::context_types::contextoid::Contextoid;
use crate::types::context_types::node_types::data::Data;
use crate::types::context_types::node_types::space::euclidean_space::EuclideanSpace;
use crate::types::context_types::node_types::space_time::euclidean_spacetime::EuclideanSpacetime;
use crate::types::context_types::node_types::symbol::base_symbol::BaseSymbol;
use crate::types::context_types::node_types::time::euclidean_time::EuclideanTime;
use crate::types::model_types::model::Model;

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
