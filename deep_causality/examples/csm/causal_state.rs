// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Display, Formatter};
use deep_causality::prelude::{Causable, CausalityError, Causaloid, NumericalValue};

#[derive(Clone)]
pub struct CausalState<'l>
{
    id: usize,
    version: usize,
    data: &'l [NumericalValue],
    causaloid: &'l Causaloid,
}

impl<'l> CausalState<'l>
{
    pub fn new
    (
        id: usize,
        version: usize,
        data: &'l [NumericalValue],
        causaloid: &'l Causaloid,
    )
        -> Self
    {
        Self { id, version, data, causaloid }
    }
}

impl<'l> CausalState<'l>
{
    pub fn eval(&self) -> Result<bool, CausalityError>
    {
        if self.causaloid.is_singleton() {
            let obs = &self.data[0];
            self.causaloid.verify_single_cause(obs)
        } else {
            self.causaloid.verify_all_causes(&self.data, None)
        }
    }
}

impl<'l> CausalState<'l>
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
    pub fn causaloid(&self) -> &'l Causaloid
    {
        self.causaloid
    }
}

impl<'l> Display for CausalState<'l>
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