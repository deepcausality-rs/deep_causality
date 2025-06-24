// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableLorentzianSpacetime, Coordinate};

impl Coordinate<f64> for AdjustableLorentzianSpacetime {
    fn dimension(&self) -> usize {
        4
    }

    fn coordinate(&self, index: usize) -> &f64 {
        match index {
            0 => &self.t,
            1 => &self.x,
            2 => &self.y,
            3 => &self.z,
            _ => panic!("AdjustableLorentzianSpacetime: index out of bounds"),
        }
    }
}
