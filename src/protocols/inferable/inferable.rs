/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use std::fmt::Debug;

use crate::prelude::{DescriptionValue, Identifiable, NumericalValue};

pub trait Inferable: Debug + Identifiable
{
    type NumericValue: PartialOrd;
    fn question(&self) -> DescriptionValue;
    fn observation(&self) -> NumericalValue;
    fn threshold(&self) -> NumericalValue;
    fn effect(&self) -> NumericalValue;
    fn target(&self) -> NumericalValue;
    fn conjoint_delta(&self) -> NumericalValue;
    fn is_inferable(&self) -> bool;
    fn is_inverse_inferable(&self) -> bool;
}
