/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{Assumption, Causaloid, Context, Datable, Identifiable, Symbolic};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use deep_causality_macros::Getters;
use std::sync::Arc;

/// Optional shared reference to a context with fully generic dimensions.
///
/// This is useful for scenarios where the context may or may not be present,
/// and needs to be shared across threads or owners.
///
/// # Type Parameters
/// - `D`: Data type (must be `Copy + Eq`)
/// - `S`: Spatial coordinate type
/// - `T`: Temporal coordinate type
/// - `ST`: Spacetime coordinate type
/// - `SYM`: Symbolic coordinate type
/// - `VS`: Value type for spatial reasoning
/// - `VT`: Value type for temporal reasoning
pub type OptionalSharedContext<D, S, T, ST, SYM, VS, VT> =
    Option<Arc<Context<D, S, T, ST, SYM, VS, VT>>>;

#[derive(Getters)]
pub struct Model<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    #[getter(name = model_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    author: String,
    description: String,
    assumptions: Option<Arc<Vec<Assumption>>>,
    causaloid: Arc<Causaloid<D, S, T, ST, SYM, VS, VT>>,
    context: OptionalSharedContext<D, S, T, ST, SYM, VS, VT>,
}

impl<D, S, T, ST, SYM, VS, VT> Model<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    pub fn new(
        id: u64,
        author: &str,
        description: &str,
        assumptions: Option<Arc<Vec<Assumption>>>,
        causaloid: Arc<Causaloid<D, S, T, ST, SYM, VS, VT>>,
        context: OptionalSharedContext<D, S, T, ST, SYM, VS, VT>,
    ) -> Self {
        Self {
            id,
            author: author.to_string(),
            description: description.to_string(),
            assumptions,
            causaloid,
            context,
        }
    }
}

impl<D, S, T, ST, SYM, VS, VT> Identifiable for Model<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn id(&self) -> u64 {
        self.id
    }
}
