/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use ultragraph::{GraphMut, GraphView};

use crate::*;

use crate::MonadicCausableGraphReasoning;
use crate::{CausalGraphIndexError, CausalityGraphError};

// Marker trait to add default impl from
#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> MonadicCausableGraphReasoning<I, O, D, S, T, ST, SYM, VS, VT>
    for CausaloidGraph<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>
where
    I: IntoEffectValue + Default,
    O: IntoEffectValue + Default,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    Causaloid<I, O, D, S, T, ST, SYM, VS, VT>: MonadicCausable<CausalMonad>,
{
}

#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> CausableGraph<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>
    for CausaloidGraph<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>
where
    I: IntoEffectValue + Default,
    O: IntoEffectValue + Default,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    Causaloid<I, O, D, S, T, ST, SYM, VS, VT>: MonadicCausable<CausalMonad>,
{
    fn is_frozen(&self) -> bool {
        self.graph.is_frozen()
    }

    fn freeze(&mut self) {
        self.graph.freeze()
    }

    fn unfreeze(&mut self) {
        self.graph.unfreeze()
    }

    fn get_graph(&self) -> &CausalGraph<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>> {
        &self.graph
    }

    fn add_root_causaloid(
        &mut self,
        value: Causaloid<I, O, D, S, T, ST, SYM, VS, VT>,
    ) -> Result<usize, CausalityGraphError> {
        match self.graph.add_root_node(value) {
            Ok(index) => Ok(index),
            Err(e) => Err(CausalityGraphError(e.to_string())),
        }
    }

    fn contains_root_causaloid(&self) -> bool {
        self.graph.contains_root_node()
    }

    fn get_root_causaloid(&self) -> Option<&Causaloid<I, O, D, S, T, ST, SYM, VS, VT>> {
        self.graph.get_root_node()
    }

    fn get_root_index(&self) -> Option<usize> {
        self.graph.get_root_index()
    }

    fn get_last_index(&self) -> Result<usize, CausalityGraphError> {
        self.graph.get_last_index().ok_or_else(|| {
            CausalityGraphError("Failed to get last index. Graph might be empty".to_string())
        })
    }

    fn add_causaloid(
        &mut self,
        value: Causaloid<I, O, D, S, T, ST, SYM, VS, VT>,
    ) -> Result<usize, CausalityGraphError> {
        match self.graph.add_node(value) {
            Ok(index) => Ok(index),
            Err(e) => Err(CausalityGraphError(e.to_string())),
        }
    }

    fn contains_causaloid(&self, index: usize) -> bool {
        self.graph.contains_node(index)
    }

    fn get_causaloid(&self, index: usize) -> Option<&Causaloid<I, O, D, S, T, ST, SYM, VS, VT>> {
        self.graph.get_node(index)
    }

    fn remove_causaloid(&mut self, index: usize) -> Result<(), CausalGraphIndexError> {
        match self.graph.remove_node(index) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError> {
        match self.graph.add_edge(a, b, 0) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    fn add_edg_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    ) -> Result<(), CausalGraphIndexError> {
        match self.graph.add_edge(a, b, weight) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.graph.contains_edge(a, b)
    }

    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError> {
        match self.graph.remove_edge(a, b) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    fn size(&self) -> usize {
        self.graph.number_nodes()
    }

    fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }

    fn clear(&mut self) {
        let _ = self.graph.clear();
    }

    fn number_edges(&self) -> usize {
        self.graph.number_edges()
    }

    fn number_nodes(&self) -> usize {
        self.graph.number_nodes()
    }
}
