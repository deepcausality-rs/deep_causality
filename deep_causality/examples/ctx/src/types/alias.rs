// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{BaseNumberType, Causaloid, Context, Model, Space, SpaceTime, Time};

use crate::types::dateoid::CustomData;

pub type CustomContext<'l> = Context<
    'l,
    CustomData,
    Space<BaseNumberType>,
    Time<BaseNumberType>,
    SpaceTime<BaseNumberType>,
    BaseNumberType,
>;
pub type CustomCausaloid<'l> = Causaloid<
    'l,
    CustomData,
    Space<BaseNumberType>,
    Time<BaseNumberType>,
    SpaceTime<BaseNumberType>,
    BaseNumberType,
>;
pub type CustomModel<'l> = Model<
    'l,
    CustomData,
    Space<BaseNumberType>,
    Time<BaseNumberType>,
    SpaceTime<BaseNumberType>,
    BaseNumberType,
>;
