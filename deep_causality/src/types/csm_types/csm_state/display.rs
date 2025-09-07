/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalState, Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
use std::fmt::{Display, Formatter};

impl<D, S, T, ST, SYM, VS, VT> Display for CausalState<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CausalState: id: {} version: {} data: {{:?}} causaloid: {} {}",
            self.id, self.version, self.data, self.causaloid,
        )
    }
}
