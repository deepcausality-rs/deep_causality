/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CentralityGraphAlgorithms, CsmGraph, GraphError, GraphView};
use std::collections::VecDeque;

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
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return Ok(Vec::new());
        }

        let mut centrality = vec![0.0; num_nodes];

        for s in 0..num_nodes {
            let (_, sigma, pred, mut stack) = self._brandes_bfs_and_path_counting(s, directed)?;

            // Accumulation
            let mut delta = vec![0.0; num_nodes];
            while let Some(w) = stack.pop() {
                for &v in &pred[w] {
                    let sigma_v = sigma[v];
                    let sigma_w = sigma[w];
                    if sigma_w == 0.0 {
                        // This should ideally not happen if paths are correctly counted,
                        // but as a safeguard against division by zero.
                        return Err(GraphError::AlgorithmError(
                            "Division by zero in sigma calculation",
                        ));
                    }
                    delta[v] += (sigma_v / sigma_w) * (1.0 + delta[w]);
                }
                if w != s {
                    centrality[w] += delta[w];
                }
            }
        }

        // Normalization
        if normalized {
            let scale = if directed {
                ((num_nodes - 1) * (num_nodes - 2)) as f64
            } else {
                ((num_nodes - 1) * (num_nodes - 2)) as f64 / 2.0
            };

            if scale > 0.0 {
                for score in centrality.iter_mut() {
                    *score /= scale;
                }
            }
        }

        Ok(centrality.into_iter().enumerate().collect())
    }

    fn pathway_betweenness_centrality(
        &self,
        pathways: &[(usize, usize)],
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError> {
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return Ok(Vec::new());
        }

        let mut centrality = vec![0.0; num_nodes];

        for &(s, t) in pathways {
            if !self.contains_node(s) || !self.contains_node(t) {
                // Skip invalid pathways
                continue;
            }

            let (dist, sigma, pred, mut stack) =
                self._brandes_bfs_and_path_counting(s, directed)?;

            // Accumulation for specific pathway (s, t)
            if dist[t].is_some() {
                // Only accumulate if t is reachable from s
                let mut delta = vec![0.0; num_nodes];
                delta[t] = 1.0;

                while let Some(w) = stack.pop() {
                    for &v in &pred[w] {
                        let sigma_v = sigma[v];
                        let sigma_w = sigma[w];
                        if sigma_w == 0.0 {
                            return Err(GraphError::AlgorithmError(
                                "Division by zero in sigma calculation for pathway",
                            ));
                        }
                        delta[v] += (sigma_v / sigma_w) * (1.0 + delta[w]);
                    }
                    if w != s {
                        centrality[w] += delta[w];
                    }
                }
            }
        }

        // Normalization
        if normalized {
            let num_pathways = pathways.len() as f64;
            if num_pathways > 0.0 {
                for score in centrality.iter_mut() {
                    *score /= num_pathways;
                }
            }
        }

        Ok(centrality.into_iter().enumerate().collect())
    }
}

impl<N, W> CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default,
{
    /// Private helper for Brandes' algorithm: BFS and path counting phase.
    /// Returns (dist, sigma, pred, stack)
    #[allow(clippy::type_complexity)]
    #[inline]
    fn _brandes_bfs_and_path_counting(
        &self,
        s: usize,
        directed: bool,
    ) -> Result<(Vec<Option<usize>>, Vec<f64>, Vec<Vec<usize>>, Vec<usize>), GraphError> {
        let num_nodes = self.number_nodes();
        let mut stack = Vec::new(); // S in Brandes' algorithm
        let mut queue = VecDeque::new();
        let mut dist = vec![None; num_nodes];
        let mut sigma = vec![0.0; num_nodes]; // Number of shortest paths
        let mut pred: Vec<Vec<usize>> = vec![Vec::new(); num_nodes]; // Predecessors on shortest paths

        dist[s] = Some(0);
        sigma[s] = 1.0;
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);

            let mut neighbors_to_process = Vec::new();
            if directed {
                let start = self.forward_edges.offsets[v];
                let end = self.forward_edges.offsets[v + 1];
                for &neighbor in &self.forward_edges.targets[start..end] {
                    neighbors_to_process.push(neighbor);
                }
            } else {
                // For undirected, consider both forward and backward neighbors
                let start_fwd = self.forward_edges.offsets[v];
                let end_fwd = self.forward_edges.offsets[v + 1];
                for &neighbor in &self.forward_edges.targets[start_fwd..end_fwd] {
                    neighbors_to_process.push(neighbor);
                }
                let start_bwd = self.backward_edges.offsets[v];
                let end_bwd = self.backward_edges.offsets[v + 1];
                for &neighbor in &self.backward_edges.targets[start_bwd..end_bwd] {
                    neighbors_to_process.push(neighbor);
                }
                neighbors_to_process.sort_unstable();
                neighbors_to_process.dedup(); // Remove duplicates if any
            }

            for &w in &neighbors_to_process {
                let v_dist = dist[v].ok_or(GraphError::AlgorithmError("Distance for v not set"))?;
                // Path discovery
                if dist[w].is_none() {
                    dist[w] = Some(v_dist + 1);
                    queue.push_back(w);
                }
                // Path counting
                if dist[w].ok_or(GraphError::AlgorithmError("Distance for w not set"))?
                    == v_dist + 1
                {
                    sigma[w] += sigma[v];
                    pred[w].push(v);
                }
            }
        }
        Ok((dist, sigma, pred, stack))
    }
}
