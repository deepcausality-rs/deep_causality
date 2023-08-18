// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Debug, Display, Formatter};

use crate::types::reasoning_types::assumption::Assumption;

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
               "Assumption: id: {}, description: {}, assumption_fn: fn(&[NumericalValue]) -> bool;, assumption_tested: {},assumption_valid: {}",
               self.id,
               self.description,
               self.assumption_tested.borrow(),
               self.assumption_valid.borrow()
        )
    }
}
