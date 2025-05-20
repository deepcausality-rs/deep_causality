// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::*;

use deep_causality_macros::{Constructor, Getters};

use crate::prelude::{
    Causable, CausalityError, Causaloid, Datable, NumericalValue, SpaceTemporal, Spatial,
    Temporable,
};

#[derive(Getters, Constructor, Clone, Debug)]
pub struct CausalState<'l, D, S, T, ST, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporable<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>
        + Clone,
{
    id: usize,
    version: usize,
    data: NumericalValue,
    causaloid: &'l Causaloid<'l, D, S, T, ST, V>,
}

impl<D, S, T, ST, V> CausalState<'_, D, S, T, ST, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporable<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>
        + Clone,
{
    pub fn eval(&self) -> Result<bool, CausalityError> {
        self.causaloid.verify_single_cause(&self.data)
    }
    pub fn eval_with_data(&self, data: &NumericalValue) -> Result<bool, CausalityError> {
        self.causaloid.verify_single_cause(data)
    }

    fn fmt_print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CausalState: \n id: {} version: {} \n data: {:?} causaloid: {:?}",
            self.id, self.version, self.data, self.causaloid,
        )
    }
}

impl<D, S, T, ST, V> Display for CausalState<'_, D, S, T, ST, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporable<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>
        + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_print(f)
    }
}
