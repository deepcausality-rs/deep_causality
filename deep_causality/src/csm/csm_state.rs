// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Debug, Display, Formatter};
use crate::prelude::{Causable, CausalityError, Causaloid, Datable, NumericalValue, SpaceTemporal, Spatial, Temporal};

#[derive(Clone, Debug)]
pub struct CausalState<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
{
    id: usize,
    version: usize,
    data: &'l [NumericalValue],
    causaloid: &'l Causaloid<'l, D, S, T, ST>,
}

impl<'l, D, S, T, ST>  CausalState<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
{
    pub fn new
    (
        id: usize,
        version: usize,
        data: &'l [NumericalValue],
        causaloid: &'l Causaloid<D, S, T, ST>,
    )
        -> Self
    {
        Self { id, version, data, causaloid }
    }
}

impl<'l, D, S, T, ST>  CausalState<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
{
    pub fn eval(&self) -> Result<bool, CausalityError>
    {
        if self.causaloid.is_singleton() {
            let obs = &self.data[0];
            self.causaloid.verify_single_cause(obs)
        } else {
            self.causaloid.verify_all_causes(self.data, None)
        }
    }
    pub fn eval_with_data(
        &self,
        data: &'l [NumericalValue],
    )
        -> Result<bool, CausalityError>
    {
        if self.causaloid.is_singleton() {
            let obs = &data[0];
            self.causaloid.verify_single_cause(obs)
        } else {
            self.causaloid.verify_all_causes(data, None)
        }
    }
}

impl<'l, D, S, T, ST>  CausalState<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
{
    pub fn id(&self) -> usize
    {
        self.id
    }
    pub fn version(&self) -> usize
    {
        self.version
    }
    pub fn data(&self) -> &'l [NumericalValue]
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
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
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