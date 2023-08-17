// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Debug, Display, Formatter};

use crate::prelude::{Causable, CausalityError, Causaloid, Datable, NumericalValue, SpaceTemporal, Spatial, Temporal};

#[derive(Clone, Debug)]
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
    pub fn new
    (
        id: usize,
        version: usize,
        data: NumericalValue,
        causaloid: &'l Causaloid<'l, D, S, T, ST>,
    )
        -> Self
    {
        assert!(causaloid.is_singleton());

        Self { id, version, data, causaloid }
    }
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

impl<'l, D, S, T, ST>  CausalState<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    pub fn id(&self) -> usize
    {
        self.id
    }
    pub fn version(&self) -> usize
    {
        self.version
    }
    pub fn data(&self) -> NumericalValue
    {
        self.data
    }
    pub fn causaloid(&self) -> &'l Causaloid<D, S, T, ST>
    {
        self.causaloid
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