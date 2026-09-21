/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Distance, NedSpace};
use deep_causality_algebra::RealField;

impl<R: RealField> Distance for NedSpace<R> {
    fn distance(&self, other: &Self) -> R {
        let dn = self.north - other.north;
        let de = self.east - other.east;
        let dd = self.down - other.down;
        (dn * dn + de * de + dd * dd).sqrt()
    }
}
