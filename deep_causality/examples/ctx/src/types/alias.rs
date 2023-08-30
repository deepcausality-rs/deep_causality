// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{
    BaseNumberType, Causaloid, Context, Model, SpaceTempoid, Spaceoid, Tempoid,
};

use crate::types::dateoid::CustomData;

pub type CustomContext<'l> = Context<
    'l,
    CustomData,
    Spaceoid<BaseNumberType>,
    Tempoid<BaseNumberType>,
    SpaceTempoid<BaseNumberType>,
    BaseNumberType,
>;
pub type CustomCausaloid<'l> = Causaloid<
    'l,
    CustomData,
    Spaceoid<BaseNumberType>,
    Tempoid<BaseNumberType>,
    SpaceTempoid<BaseNumberType>,
    BaseNumberType,
>;
pub type CustomModel<'l> = Model<
    'l,
    CustomData,
    Spaceoid<BaseNumberType>,
    Tempoid<BaseNumberType>,
    SpaceTempoid<BaseNumberType>,
    BaseNumberType,
>;
