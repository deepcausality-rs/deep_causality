// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::{Debug, Display, Formatter};

use deep_causality_macros::{Constructor, Getters};

use crate::prelude::{Causable, CausalityError, Causaloid, Datable, NumericalValue, SpaceTemporal, Spatial, Temporal};

#[derive(Getters, Constructor, Clone, Debug)]
pub struct CausalState<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    id: usize,
    version: usize,
    data: NumericalValue,
    causaloid: &'l Causaloid<'l, D, S, T, ST>,
}

impl<'l, D, S, T, ST>  CausalState<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    pub fn eval(&self) -> Result<bool, CausalityError>
    {
        self.causaloid.verify_single_cause(&self.data)
    }
    pub fn eval_with_data(
        &self,
        data: &NumericalValue,
    )
        -> Result<bool, CausalityError>
    {
        self.causaloid.verify_single_cause(data)

    }
}


impl<'l, D, S, T, ST> Display for CausalState<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        write!(f,
               "CausalState: \n id: {} version: {} \n data: {:?} causaloid: {}",
               self.id,
               self.version,
               self.data,
               self.causaloid,
        )
    }
}