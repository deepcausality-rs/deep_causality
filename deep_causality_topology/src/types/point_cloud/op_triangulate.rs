/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{PointCloud, Simplex, SimplicialComplex, Skeleton, TopologyError};
use alloc::collections::BTreeSet;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_sparse::CsrMatrix;

// Helper function to calculate Euclidean distance between two points
fn euclidean_distance(p1_slice: &[f64], p2_slice: &[f64]) -> f64 {
    p1_slice
        .iter()
        .zip(p2_slice.iter())
        .map(|(&x1, &x2)| (x1 - x2).powi(2))
        .sum::<f64>()
        .sqrt()
}

impl<T> PointCloud<T> {
    /// Converts the `PointCloud` into a `SimplicialComplex` using a Vietoris-Rips filtration.
    /// This method infers connectivity based on the `radius` parameter:
    /// any two points within `radius` distance form a 1-simplex (edge).
    /// Higher-order simplices are then formed by sets of points that are pairwise within `radius`.
    ///
    /// # Implementation Details:
    /// - **0-simplices**: Each point in the cloud forms a 0-simplex (vertex).
    /// - **1-simplices**: An edge (1-simplex) is created between any two points
    ///   whose Euclidean distance is less than or equal to `radius`.
    /// - **Higher-order simplices**: Higher-order simplices (k-simplices for k > 1) are generated
    ///   iteratively. A k-simplex is formed by k+1 vertices where all pairs are connected by an edge.
    ///   This is achieved by finding cliques in the graph formed by the 1-skeleton. The process
    ///   terminates when no new simplices can be formed in a given dimension.
    /// - **Boundary Operators**: Boundary matrices (B_k) are computed for each dimension. The coefficient
    ///   for each (k-1)-face in the boundary of a k-simplex is determined by its orientation, using
    ///   alternating sums based on the lexicographical ordering of vertices.
    /// - **Coboundary Operators**: Coboundary matrices (C_k) are derived by transposing the corresponding
    ///   higher-dimensional boundary matrices (C_k is the transpose of B_{k+1}).
    pub fn triangulate(&self, radius: f64) -> Result<SimplicialComplex, TopologyError> {
        if self.is_empty() {
            return Err(TopologyError::PointCloudError(
                "Cannot triangulate an empty point cloud".to_string(),
            ));
        }
        if radius <= 0.0 {
            return Err(TopologyError::InvalidInput(
                "Triangulation radius must be positive".to_string(),
            ));
        }

        let num_points = self.len();
        let point_dim = self.points.shape()[1]; // Dimensionality of the points
        let points_data = self.points.as_slice(); // Flat slice of all point coordinates

        // 0-SKELETON: Each point is a 0-simplex
        let mut zero_simplices = Vec::with_capacity(num_points);
        for i in 0..num_points {
            zero_simplices.push(Simplex::new(vec![i]));
        }
        let zero_skeleton = Skeleton::new(0, zero_simplices);

        // ADJACENCY MATRIX: To store 1-simplices (edges)
        let mut adj_matrix = vec![vec![false; num_points]; num_points];
        let mut one_simplices = BTreeSet::new(); // Use BTreeSet to automatically handle uniqueness and sorting

        // Iterate through all unique pairs of points
        for i in 0..num_points {
            let p1_coords = &points_data[i * point_dim..(i + 1) * point_dim];
            for j in (i + 1)..num_points {
                let p2_coords = &points_data[j * point_dim..(j + 1) * point_dim];

                if euclidean_distance(p1_coords, p2_coords) <= radius {
                    adj_matrix[i][j] = true;
                    adj_matrix[j][i] = true;
                    // Add the 1-simplex (edge)
                    one_simplices.insert(Simplex::new(vec![i, j]));
                }
            }
        }
        let one_skeleton = Skeleton::new(1, one_simplices.into_iter().collect());

        // Skeletons collected (for now, just 0 and 1)
        let mut skeletons = vec![zero_skeleton, one_skeleton];

        // GENERATE HIGHER-ORDER SIMPLICES (k-simplices for k > 1)
        // This is done by finding cliques in the graph formed by the 1-skeleton.
        // A k-simplex is formed by k+1 vertices where all pairs are connected by an edge.
        let mut k = 2;
        loop {
            let prev_skeleton_idx = k - 1;
            if prev_skeleton_idx >= skeletons.len() {
                // This means no simplices were found in the previous dimension, so we stop.
                break;
            }
            let prev_simplices = &skeletons[prev_skeleton_idx].simplices;
            if prev_simplices.is_empty() {
                // No (k-1)-simplices, so no k-simplices can be formed.
                break;
            }

            let mut current_k_simplices = BTreeSet::new();

            // Iterate over each (k-1)-simplex
            for prev_simplex in prev_simplices {
                // Try to extend it with every possible vertex
                for (v_candidate, adj_row_for_v_candidate) in adj_matrix.iter().enumerate() {
                    // Check if v_candidate is already part of the prev_simplex
                    if prev_simplex.vertices().contains(&v_candidate) {
                        continue;
                    }

                    // Check if v_candidate is connected to ALL vertices in prev_simplex
                    let mut is_connected_to_all = true;
                    for &u_vertex in prev_simplex.vertices() {
                        if !adj_row_for_v_candidate[u_vertex] {
                            // Use adj_row_for_v_candidate instead of adj_matrix[v_candidate]
                            is_connected_to_all = false;
                            break;
                        }
                    }

                    if is_connected_to_all {
                        // Form a new k-simplex
                        let mut new_simplex_vertices = prev_simplex.vertices().clone();
                        new_simplex_vertices.push(v_candidate);
                        new_simplex_vertices.sort_unstable(); // Canonical representation
                        new_simplex_vertices.dedup(); // Remove duplicates (shouldn't be any if logic is correct)

                        if new_simplex_vertices.len() == k + 1 {
                            // Ensure it's a k-simplex
                            current_k_simplices.insert(Simplex::new(new_simplex_vertices));
                        }
                    }
                }
            }

            if current_k_simplices.is_empty() {
                break; // No new k-simplices found, terminate
            }

            skeletons.push(Skeleton::new(k, current_k_simplices.into_iter().collect()));
            k += 1;
        }

        let max_dim = skeletons.len() - 1;

        // BOUNDARY OPERATORS (B_k)
        // B_k: maps k-simplices to (k-1)-simplices. Rows are (k-1)-simplices, columns are k-simplices.
        let mut boundary_operators = Vec::with_capacity(max_dim + 1);

        // B_0 maps 0-simplices to -1 dimension, which is trivial (all zeros)
        boundary_operators
            .push(CsrMatrix::from_triplets(0, skeletons[0].simplices.len(), &[]).unwrap());

        for k_dim in 1..=max_dim {
            let num_prev_simplices = skeletons[k_dim - 1].simplices.len();
            let num_curr_simplices = skeletons[k_dim].simplices.len();
            let mut triplets: Vec<(usize, usize, i8)> = Vec::new();

            // For each k-simplex (column in the matrix)
            for (col_idx, s_k) in skeletons[k_dim].simplices().iter().enumerate() {
                // For each (k-1)-face of the k-simplex
                for i in 0..=k_dim {
                    let mut face_vertices = s_k.vertices().clone();
                    face_vertices.remove(i); // Remove the i-th vertex to form a face

                    let face_simplex = Simplex::new(face_vertices);

                    // Find the row index of this (k-1)-face in the (k-1)-skeleton
                    let row_idx = skeletons[k_dim - 1]
                        .get_index(&face_simplex)
                        .ok_or(TopologyError::SimplexNotFound)?;

                    // Determine the coefficient based on orientation
                    let coefficient = if i % 2 == 0 { 1 } else { -1 };

                    triplets.push((row_idx, col_idx, coefficient));
                }
            }
            boundary_operators.push(
                CsrMatrix::from_triplets(num_prev_simplices, num_curr_simplices, &triplets)
                    .unwrap(),
            );
        }

        // COBOUNDARY OPERATORS (C_k)
        // C_k is the transpose of B_{k+1}
        let mut coboundary_operators = Vec::with_capacity(max_dim + 1);

        for k_dim in 0..max_dim {
            // C_k is the transpose of B_{k+1}
            coboundary_operators.push(boundary_operators[k_dim + 1].transpose());
        }
        // For the highest dimension, coboundary maps to an empty (max_dim+1) dimension
        coboundary_operators
            .push(CsrMatrix::from_triplets(0, skeletons[max_dim].simplices.len(), &[]).unwrap());

        Ok(SimplicialComplex::new(
            skeletons,
            boundary_operators,
            coboundary_operators,
        ))
    }
}
