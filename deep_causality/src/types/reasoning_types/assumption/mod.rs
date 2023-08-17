// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::cell::RefCell;

use crate::prelude::{DescriptionValue, EvalFn, IdentificationValue};

mod identifiable;
mod assumable;
mod debug;


#[derive(Clone)]
pub struct Assumption
{
    id: IdentificationValue,
    description: DescriptionValue,
    assumption_fn: EvalFn,
    assumption_tested: RefCell<bool>,
    assumption_valid: RefCell<bool>,
}

// Constructor
impl Assumption
{
    pub fn new(id: IdentificationValue, description: DescriptionValue, assumption_fn: EvalFn,
    ) -> Self
    {
        Self {
            id,
            description,
            assumption_fn,
            assumption_tested: RefCell::from(false),
            assumption_valid: RefCell::from(false),
        }
    }
}
