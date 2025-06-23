// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{
    BaseSymbol, CausalityError, Causaloid, CausaloidGraph, Context, Contextoid, Data,
    EuclideanSpace, EuclideanSpacetime, EuclideanTime, SymbolicResult,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Type aliases
pub type IdentificationValue = u64;
pub type NumericalValue = f64;
pub type DescriptionValue = String;

// Thread safe Interior mutability in Rust
// https://ricardomartins.cc/2016/06/25/interior-mutability-thread-safety
pub type ArcRWLock<T> = Arc<RwLock<T>>;

// Fn aliases for assumable, assumption, & assumption collection
pub type EvalFn = fn(&[NumericalValue]) -> bool;

pub type SymbolicRepr = String;
pub type SymbolicCausalFn = fn(SymbolicRepr) -> Result<SymbolicResult, CausalityError>;

pub type ProbabilisticCausalFn = fn(NumericalValue) -> Result<NumericalValue, CausalityError>; // Bayes update, etc.

// Fn aliases for causal function with and without context
pub type CausalFn = fn(NumericalValue) -> Result<bool, CausalityError>;

pub type ContextualCausalDataFn<'l, D, S, T, ST, SYM, VS, VT> =
    fn(NumericalValue, &'l Context<D, S, T, ST, SYM, VS, VT>) -> Result<bool, CausalityError>;

pub type ContextualCausalFn<'l, D, S, T, ST, SYM, VS, VT> =
    fn(&'l Context<D, S, T, ST, SYM, VS, VT>) -> Result<bool, CausalityError>;

// Default type aliases for basic causaloids

pub type BaseNumberType = u64;
pub type BaseFloatType = f64;

pub type BaseCausaloid<'l> = Causaloid<
    'l,
    Data<BaseNumberType>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    BaseFloatType,
    BaseFloatType,
>;

pub type BaseCausaloidVec<'l> = Vec<
    Causaloid<
        'l,
        Data<BaseNumberType>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        BaseFloatType,
        BaseFloatType,
    >,
>;

pub type BaseCausalMap<'l> = HashMap<
    usize,
    Causaloid<
        'l,
        Data<BaseNumberType>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        BaseFloatType,
        BaseFloatType,
    >,
>;

pub type BaseCausalGraph<'l> = CausaloidGraph<
    Causaloid<
        'l,
        Data<BaseNumberType>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        BaseFloatType,
        BaseFloatType,
    >,
>;

// Default type alias for basic context. It's used in tests
pub type BaseContext = Context<
    Data<BaseNumberType>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    BaseFloatType,
    BaseFloatType,
>;

pub type BaseContextoid = Contextoid<
    Data<BaseNumberType>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    BaseFloatType,
    BaseFloatType,
>;
