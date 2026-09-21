/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::{Contextoid, ContextoidType, Datable};

impl<D, S, T, ST, VS, VT> Contextoid<D, S, T, ST, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    VS: Clone,
    VT: Clone,
{
    pub fn vertex_type(&self) -> &ContextoidType<D, S, T, ST, VS, VT> {
        &self.vertex_type
    }
}
