// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::collections::HashMap;

use deep_causality::prelude::{
    Causaloid, CausaloidGraph, Dataoid, SpaceTempoid, Spaceoid, Tempoid,
};

pub(crate) type DefaultType = u64;

pub(crate) type Causal<'l> = Causaloid<
    'l,
    Dataoid<DefaultType>,
    Spaceoid<DefaultType>,
    Tempoid<DefaultType>,
    SpaceTempoid<DefaultType>,
    DefaultType,
>;

pub(crate) type CausalVector = Vec<
    Causaloid<
        'static,
        Dataoid<DefaultType>,
        Spaceoid<DefaultType>,
        Tempoid<DefaultType>,
        SpaceTempoid<DefaultType>,
        DefaultType,
    >,
>;

pub(crate) type CausalGraph<'l> = CausaloidGraph<
    Causaloid<
        'l,
        Dataoid<DefaultType>,
        Spaceoid<DefaultType>,
        Tempoid<DefaultType>,
        SpaceTempoid<DefaultType>,
        DefaultType,
    >,
>;

pub(crate) type CausalMap = HashMap<
    usize,
    Causaloid<
        'static,
        Dataoid<DefaultType>,
        Spaceoid<DefaultType>,
        Tempoid<DefaultType>,
        SpaceTempoid<DefaultType>,
        DefaultType,
    >,
>;
