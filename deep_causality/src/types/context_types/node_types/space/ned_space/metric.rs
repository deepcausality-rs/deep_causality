/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{Metric, NedSpace};

impl Metric<f64> for NedSpace {
    fn distance(&self, other: &Self) -> f64 {
        let dn = self.north - other.north;
        let de = self.east - other.east;
        let dd = self.down - other.down;
        (dn * dn + de * de + dd * dd).sqrt()
    }
}
