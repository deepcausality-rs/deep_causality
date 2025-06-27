/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::prelude::{
    Causaloid, CausaloidGraph, Context, Contextoid, Data, FloatType, Model, NumberType, SpaceKind,
    SpaceTimeKind, TimeKind,
};
use crate::types::context_types::node_types::symbol::symbol_kind::SymbolKind;
use std::collections::HashMap;

pub type UniformModel =
    Model<Data<NumberType>, SpaceKind, TimeKind, SpaceTimeKind, SymbolKind, FloatType, FloatType>;

pub type UniformCausaloid = Causaloid<
    Data<NumberType>,
    SpaceKind,
    TimeKind,
    SpaceTimeKind,
    SymbolKind,
    FloatType,
    FloatType,
>;

pub type UniformCausaloidVec = Vec<
    Causaloid<
        Data<NumberType>,
        SpaceKind,
        TimeKind,
        SpaceTimeKind,
        SymbolKind,
        FloatType,
        FloatType,
    >,
>;

pub type UniformCausalMap = HashMap<
    usize,
    Causaloid<
        Data<NumberType>,
        SpaceKind,
        TimeKind,
        SpaceTimeKind,
        SymbolKind,
        FloatType,
        FloatType,
    >,
>;

pub type UniformCausalGraph = CausaloidGraph<
    Causaloid<
        Data<NumberType>,
        SpaceKind,
        TimeKind,
        SpaceTimeKind,
        SymbolKind,
        FloatType,
        FloatType,
    >,
>;

pub type UniformContext =
    Context<Data<NumberType>, SpaceKind, TimeKind, SpaceTimeKind, SymbolKind, FloatType, FloatType>;

pub type UniformContextoid = Contextoid<
    Data<NumberType>,
    SpaceKind,
    TimeKind,
    SpaceTimeKind,
    SymbolKind,
    FloatType,
    FloatType,
>;
