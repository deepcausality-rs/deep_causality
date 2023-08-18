// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

impl<'l, D, S, T, ST> Display for Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}


impl<'l, D, S, T, ST> Debug for Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}


impl<'l, D, S, T, ST> Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Causaloid id: {} \n Causaloid type: {} \n description: {} is active: {} has context: {}",
               self.id,
               self.causal_type,
               self.description,
               self.is_active(),
               self.has_context,
        )
    }
}