// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{
    CausalityError, Causaloid, Context, Dataoid, SpaceTempoid, Spaceoid, Tempoid,
};

// Type aliases
pub type IdentificationValue = u64;
pub type NumericalValue = f64;
pub type DescriptionValue = String;

// Fn aliases for assumable, assumption, & assumption collection
pub type EvalFn = fn(&[NumericalValue]) -> bool;

// Fn aliases for causal function with and without context
pub type CausalFn = fn(NumericalValue) -> Result<bool, CausalityError>;
pub type ContextualCausalFn<'l, D, S, T, ST, V> =
    fn(NumericalValue, &'l Context<D, S, T, ST, V>) -> Result<bool, CausalityError>;

// Default type aliases for basic causaloids

type BaseNumberType = u64;
pub type BaseCausaloidVec<'l> = Vec<
    Causaloid<
        'l,
        Dataoid<BaseNumberType>,
        Spaceoid<BaseNumberType>,
        Tempoid<BaseNumberType>,
        SpaceTempoid<BaseNumberType>,
        BaseNumberType,
    >,
>;

pub type BaseCausaloid<'l> = Causaloid<
    'l,
    Dataoid<BaseNumberType>,
    Spaceoid<BaseNumberType>,
    Tempoid<BaseNumberType>,
    SpaceTempoid<BaseNumberType>,
    BaseNumberType,
>;

// Default type alias for basic context. It's used in tests
pub type BaseContext<'l> = Context<
    'l,
    Dataoid<BaseNumberType>,
    Spaceoid<BaseNumberType>,
    Tempoid<BaseNumberType>,
    SpaceTempoid<BaseNumberType>,
    BaseNumberType,
>;
