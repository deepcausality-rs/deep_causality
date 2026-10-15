/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::storage::graph_csm::{
    graph_csm_algo_articulation_points::articulation_points_impl,
    graph_csm_algo_biconnected_components::biconnected_components_impl,
    graph_csm_algo_bridges::bridges_impl,
    graph_csm_algo_centrality::{betweenness_centrality_impl, pathway_betweenness_centrality_impl},
    graph_csm_algo_pathfinder::{
        is_reachable_impl, shortest_path_impl, shortest_path_len_impl, shortest_weighted_path_impl,
    },
    graph_csm_algo_structural::strongly_connected_components_impl,
    graph_csm_algo_topological::{find_cycle_impl, has_cycle_impl, topological_sort_impl},
};
use crate::{
    CentralityGraphAlgorithms, EdgeKey, GraphAlgorithms, GraphError, PathfindingGraphAlgorithms,
    Projection, ProjectionView, StructuralGraphAlgorithms, TopologicalGraphAlgorithms,
};
use alloc::vec::Vec;

// Every algorithm runs the body the frozen graph runs, over this view's adjacency: the
// projection's nodes and edges, counted under its mode.

impl<N, W, P> GraphAlgorithms<N, W> for ProjectionView<'_, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
}

impl<N, W, P> TopologicalGraphAlgorithms<N, W> for ProjectionView<'_, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    fn find_cycle(&self) -> Result<Option<Vec<usize>>, GraphError> {
        find_cycle_impl(self)
    }

    fn has_cycle(&self) -> Result<bool, GraphError> {
        has_cycle_impl(self)
    }

    fn topological_sort(&self) -> Result<Option<Vec<usize>>, GraphError> {
        topological_sort_impl(self)
    }
}

impl<N, W, P> PathfindingGraphAlgorithms<N, W> for ProjectionView<'_, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    fn is_reachable(&self, start_index: usize, stop_index: usize) -> Result<bool, GraphError> {
        is_reachable_impl(self, start_index, stop_index)
    }

    fn shortest_path_len(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<usize>, GraphError> {
        shortest_path_len_impl(self, start_index, stop_index)
    }

    fn shortest_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<Vec<usize>>, GraphError> {
        shortest_path_impl(self, start_index, stop_index)
    }

    fn shortest_weighted_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<(Vec<usize>, W)>, GraphError>
    where
        W: Copy + Ord + Default + core::ops::Add<Output = W>,
    {
        shortest_weighted_path_impl(self, start_index, stop_index)
    }
}

impl<N, W, P> StructuralGraphAlgorithms<N, W> for ProjectionView<'_, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    fn strongly_connected_components(&self) -> Result<Vec<Vec<usize>>, GraphError> {
        strongly_connected_components_impl(self)
    }

    fn articulation_points(&self) -> Result<Vec<usize>, GraphError> {
        articulation_points_impl(self)
    }

    fn bridges(&self) -> Result<Vec<(usize, usize)>, GraphError> {
        bridges_impl(self)
    }

    fn biconnected_components(&self) -> Result<Vec<Vec<usize>>, GraphError> {
        biconnected_components_impl(self)
    }
}

impl<N, W, P> CentralityGraphAlgorithms<N, W> for ProjectionView<'_, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    fn betweenness_centrality(
        &self,
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError> {
        betweenness_centrality_impl(self, directed, normalized)
    }

    fn pathway_betweenness_centrality(
        &self,
        pathways: &[(usize, usize)],
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError> {
        pathway_betweenness_centrality_impl(self, pathways, directed, normalized)
    }
}
