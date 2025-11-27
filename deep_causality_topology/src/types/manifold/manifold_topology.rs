/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Manifold, ManifoldTopology, SimplicialTopology};

impl ManifoldTopology for Manifold {
    fn is_oriented(&self) -> bool {
        // Check consistency of orientation using boundary operators.
        // For the highest dimension n, the boundary of every n-simplex is a sum of (n-1)-simplices.
        // In the boundary matrix (rows=(n-1)-simplices, cols=n-simplices),
        // each row represents an (n-1)-face.
        // For an oriented manifold:
        // - Internal faces must be shared by exactly 2 n-simplices with opposite induced orientation (+1, -1). Sum = 0.
        // - Boundary faces must be shared by exactly 1 n-simplex. Sum = +/- 1.
        // - Any other sum (e.g. 2, -2, 3) indicates non-orientability or non-manifold branching.

        let max_dim = self.max_simplex_dimension();
        if max_dim == 0 {
            return true; // Points are oriented
        }

        // Boundary operator from max_dim to max_dim-1
        // boundary_operators indices: 0->(0->-1?), 1->(1->0), ... k->(k->k-1)
        // Usually boundary_operators[k] maps k-chains to (k-1)-chains.
        // We need boundary_operators[max_dim].
        if let Some(boundary_op) = self.complex.boundary_operators.get(max_dim) {
            let rows = boundary_op.shape().0;
            let row_indices = boundary_op.row_indices();
            let values = boundary_op.values();

            for r in 0..rows {
                let start = row_indices[r];
                let end = row_indices[r + 1];
                let mut sum: i8 = 0;
                for i in start..end {
                    sum += values[i];
                }

                // Sum must be 0 (internal) or 1/-1 (boundary).
                // If sum is anything else (e.g. 2, -2), orientation is inconsistent.
                if sum.abs() > 1 {
                    return false;
                }
            }
        }
        true
    }

    fn satisfies_link_condition(&self) -> bool {
        // The link of every vertex must have the Euler characteristic of a sphere (interior) or disk (boundary).
        // chi(S^{n-1}) = 1 + (-1)^{n-1}
        // chi(D^{n-1}) = 1
        //
        // Algorithm:
        // For each vertex v:
        //   Calculate chi(Link(v)) = Sum_{sigma in Star(v), v in sigma} (-1)^(dim(sigma)-1)
        //   Check if chi matches sphere or disk.

        let max_dim = self.max_simplex_dimension();
        if max_dim == 0 {
            return true; // 0-manifold (points) satisfies link condition trivially?
        }

        let num_vertices = self.complex.skeletons.first().map(|s| s.simplices.len()).unwrap_or(0);
        let sphere_chi = 1 + if (max_dim - 1) % 2 == 0 { 1 } else { -1 };
        let disk_chi = 1;

        for v_idx in 0..num_vertices {
            let mut link_chi: isize = 0;

            // Iterate over all skeletons to find simplices containing v_idx
            for skeleton in &self.complex.skeletons {
                let dim = skeleton.dim;
                if dim == 0 { continue; } // Vertices don't contribute to their own link's chi in this formula

                for simplex in &skeleton.simplices {
                    // Check if simplex contains vertex v_idx
                    // Simplices are sorted, so binary search could work, but linear scan is O(dim).
                    if simplex.vertices.contains(&v_idx) {
                        let term = if (dim - 1) % 2 == 0 { 1 } else { -1 };
                        link_chi += term;
                    }
                }
            }

            if link_chi != sphere_chi && link_chi != disk_chi {
                return false;
            }
        }

        true
    }

    fn euler_characteristic(&self) -> isize {
        // The Euler characteristic is the alternating sum of the number of simplices of each dimension.
        // chi = sum_{i=0}^n (-1)^i * c_i
        let mut chi: isize = 0;
        for skeleton in &self.complex.skeletons {
            let count = skeleton.simplices.len() as isize;
            if skeleton.dim % 2 == 0 {
                chi += count;
            } else {
                chi -= count;
            }
        }
        chi
    }

    fn has_boundary(&self) -> bool {
        // A manifold has a boundary if any (n-1)-face is incident to exactly one n-simplex.
        // In the boundary matrix, this means the row has exactly 1 non-zero entry.

        let max_dim = self.max_simplex_dimension();
        if max_dim == 0 {
            return false; // Points don't have boundary in this context
        }

        if let Some(boundary_op) = self.complex.boundary_operators.get(max_dim) {
            let rows = boundary_op.shape().0;
            let row_indices = boundary_op.row_indices();

            for r in 0..rows {
                let start = row_indices[r];
                let end = row_indices[r + 1];
                let count = end - start;

                if count == 1 {
                    return true;
                }
            }
        }
        false
    }
}
