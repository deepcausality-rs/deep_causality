// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

impl<'l, D, S, T, ST> PartialEq for Causaloid<'l, D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporable + Clone,
    ST: SpaceTemporal + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
