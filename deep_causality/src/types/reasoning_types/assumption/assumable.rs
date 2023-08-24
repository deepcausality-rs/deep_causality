// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{Assumable, DescriptionValue, EvalFn, NumericalValue};
use crate::types::reasoning_types::assumption::Assumption;

impl Assumable for Assumption {
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
