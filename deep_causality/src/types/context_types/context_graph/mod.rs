/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashMap;

use ultragraph::prelude::*;

use crate::prelude::*;

mod contextuable_graph;
mod debug;
mod extendable_contextuable_graph;
mod identifiable;
mod indexable_time;

type ExtraContext<D, S, T, ST, SYM, VS, VT> = UltraGraph<Contextoid<D, S, T, ST, SYM, VS, VT>>;

type ExtraContextMap<D, S, T, ST, SYM, VS, VT> =
    HashMap<u64, ExtraContext<D, S, T, ST, SYM, VS, VT>>;

#[allow(clippy::type_complexity)]
pub struct Context<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    id: ContextId,
    name: String,
    base_context: UltraGraph<Contextoid<D, S, T, ST, SYM, VS, VT>>,
    id_to_index_map: HashMap<ContextoidId, usize>,
    extra_contexts: Option<ExtraContextMap<D, S, T, ST, SYM, VS, VT>>,
    number_of_extra_contexts: u64,
    extra_context_id: u64,
    current_index_map: HashMap<usize, usize>,
    previous_index_map: HashMap<usize, usize>,
}

impl<D, S, T, ST, SYM, VS, VT> Context<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Creates a new context with the given node capacity.
    pub fn with_capacity(id: ContextId, name: &str, capacity: usize) -> Self {
        Self {
            id,
            name: name.to_string(),
            base_context: ultragraph::new_with_matrix_storage(capacity),
            id_to_index_map: HashMap::new(),
            extra_contexts: None,
            number_of_extra_contexts: 0,
            extra_context_id: 0,
            current_index_map: HashMap::new(),
            previous_index_map: HashMap::new(),
        }
    }

    /// Returns the name of the context.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Sets the name of the context.
    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }
}
