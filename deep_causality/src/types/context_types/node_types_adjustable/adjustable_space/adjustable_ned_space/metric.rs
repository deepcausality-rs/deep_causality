// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{Metric, AdjustableNedSpace};
    
impl Metric<f64> for AdjustableNedSpace {
    fn distance(&self, other: &Self) -> f64 {
        let dn = self.north - other.north;
        let de = self.east - other.east;
        let dd = self.down - other.down;
        (dn * dn + de * de + dd * dd).sqrt()
    }
}