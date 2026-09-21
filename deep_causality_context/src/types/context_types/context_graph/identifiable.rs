/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::{Context, Datable};
use deep_causality_core::Identifiable;

#[allow(clippy::type_complexity)]
impl<D, S, T, ST> Identifiable for Context<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    /// Returns the id of the context.
    fn id(&self) -> ContextoidId {
        self.id
    }
}
