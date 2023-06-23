/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};

use crate::prelude::{Assumable, DescriptionValue, EvalFn, Identifiable, IdentificationValue, NumericalValue};

#[derive(Clone)]
pub struct Assumption
{
    id: IdentificationValue,
    description: DescriptionValue,
    assumption_fn: EvalFn,
    assumption_tested: RefCell<bool>,
    assumption_valid: RefCell<bool>,
}

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


impl Identifiable for Assumption
{
    fn id(&self) -> IdentificationValue {
        self.id
    }
}


impl Assumable for Assumption
{
    fn description(&self) -> DescriptionValue {
        self.description.to_string() as DescriptionValue
    }

    fn assumption_fn(&self) -> EvalFn {
        self.assumption_fn
    }

    fn assumption_tested(&self) -> bool {
        *self.assumption_tested.borrow()
    }

    fn assumption_valid(&self) -> bool {
        *self.assumption_valid.borrow()
    }

    fn verify_assumption(&self, data: &[NumericalValue]) -> bool {
        let res = (self.assumption_fn)(data);
        // int. mutability: https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
        *self.assumption_tested.borrow_mut() = true;

        if res {
            *self.assumption_valid.borrow_mut() = true;
        }
        res
    }
}


impl Debug for Assumption
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_write(f)
    }
}


impl Display for Assumption
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_write(f)
    }
}


impl Assumption
{
    // derive Debug isn't general enough to cover function pointers hence the function signature.
    fn fmt_write(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "Assumption:\n id: {},\n description: {},\n assumption_fn: fn(&[NumericalValue]) -> bool;,\n assumption_tested: {},\n assumption_valid: {}",
               self.id,
               self.description,
               self.assumption_tested.borrow(),
               self.assumption_valid.borrow()
        )
    }
}

