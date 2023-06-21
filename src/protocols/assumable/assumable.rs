/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use crate::prelude::{DescriptionValue, EvalFn, Identifiable, NumericalValue};

pub trait Assumable: Identifiable {
    fn description(&self) -> DescriptionValue;
    fn assumption_fn(&self) -> EvalFn;
    fn assumption_tested(&self) -> bool;
    fn assumption_valid(&self) -> bool;
    fn verify_assumption(&self, data: &[NumericalValue]) -> bool;
}
