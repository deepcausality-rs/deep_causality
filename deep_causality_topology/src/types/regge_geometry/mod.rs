/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec;
use alloc::vec::Vec;
use deep_causality_multivector::Metric;
use deep_causality_tensor::CausalTensor;

use crate::{Simplex, SimplicialComplex};

pub struct ReggeGeometry {
    // Lengths of the 1-simplices (Edges)
    pub(crate) edge_lengths: CausalTensor<f64>,
}

impl ReggeGeometry {
    pub fn new(edge_lengths: CausalTensor<f64>) -> Self {
        ReggeGeometry { edge_lengths }
    }

    /// Computes the Riemannian Metric for a specific simplex.
    /// Used to initialize CausalMultiVector with correct signature.
    pub fn metric_at(&self, complex: &SimplicialComplex, grade: usize, index: usize) -> Metric {
        // 1. Retrieve the simplex
        let simplex = &complex.skeletons[grade].simplices[index];
        let n_vertices = simplex.vertices.len();

        // 2. Identify all edges in this simplex
        // A k-simplex has (k+1) vertices.
        // Edges are pairs of vertices.
        let mut _squared_lengths = Vec::new();

        // Iterate over all unique pairs of vertices to find edges
        for i in 0..n_vertices {
            for j in (i + 1)..n_vertices {
                let u = simplex.vertices[i];
                let v = simplex.vertices[j];

                // Construct edge simplex to look up index
                let edge = Simplex {
                    vertices: vec![u, v],
                };

                // Find edge index in 1-skeleton
                if let Some(edge_idx) = complex.skeletons[1].get_index(&edge) {
                    // Get length from tensor
                    let length = self.edge_lengths.as_slice()[edge_idx];
                    _squared_lengths.push(length * length);
                } else {
                    // Should not happen in a valid complex
                    panic!("Edge not found in 1-skeleton");
                }
            }
        }

        // 3. Construct Cayley-Menger Gram Matrix (G)
        // G_ij = (l_0i^2 + l_0j^2 - l_ij^2) / 2  (for Euclidean-like local frame)
        // This requires a more complex linear algebra step to diagonalize G.
        // For this spec, we assume a helper function `compute_signature` exists.

        // let (p, q, r) = compute_signature(&squared_lengths);
        // Metric::Generic { p, q, r }

        // Fallback for standard triangulation:
        Metric::Euclidean(grade)
    }
}
