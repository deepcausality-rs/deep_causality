/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::{Contextoid, Datable};
use deep_causality_core::Identifiable;

impl<D, S, T, ST, VS, VT> Identifiable for Contextoid<D, S, T, ST, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    VS: Clone,
    VT: Clone,
{
    fn id(&self) -> u64 {
        self.id
    }
}
