/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::manifold::manifold_checks;
use crate::{Manifold, ManifoldTopology};

impl<T> ManifoldTopology for Manifold<T> {
    fn is_oriented(&self) -> bool {
        manifold_checks::is_oriented(&self.complex)
    }

    fn satisfies_link_condition(&self) -> bool {
        manifold_checks::satisfies_link_condition(&self.complex)
    }

    fn euler_characteristic(&self) -> isize {
        manifold_checks::euler_characteristic(&self.complex)
    }

    fn has_boundary(&self) -> bool {
        manifold_checks::has_boundary(&self.complex)
    }
}
