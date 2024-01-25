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
        *self.assumption_tested.read().unwrap()
    }

    fn assumption_valid(&self) -> bool {
        *self.assumption_valid.read().unwrap()
    }

    fn verify_assumption(&self, data: &[NumericalValue]) -> bool {
        let res = (self.assumption_fn)(data);
        let mut guard_tested = self.assumption_tested.write().unwrap();
        *guard_tested = true;

        if res {
            let mut guard_valid = self.assumption_valid.write().unwrap();
            *guard_valid = true;
        }
        res
    }
}
