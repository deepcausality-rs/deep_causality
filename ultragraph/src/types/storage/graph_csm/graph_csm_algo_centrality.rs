/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CentralityGraphAlgorithms, CsmGraph, GraphError};

impl<N, W> CentralityGraphAlgorithms<N, W> for CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default,
{
    fn betweenness_centrality(
        &self,
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError> {
        todo!()
    }

    fn pathway_betweenness_centrality(
        &self,
        pathways: &[(usize, usize)],
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError> {
        todo!()
    }
}
