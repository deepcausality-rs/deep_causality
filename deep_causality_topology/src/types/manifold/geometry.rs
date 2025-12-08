/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Manifold, Simplex, TopologyError};
use deep_causality_tensor::{CausalTensor, CausalTensorError};

impl<T> Manifold<T> {
    /// Computes the squared volume of a k-simplex using the Cayley-Menger determinant.
    /// The volume is derived from the squared lengths of the edges of the simplex.
    pub fn simplex_volume_squared(&self, simplex: &Simplex) -> Result<f64, TopologyError> {
        let k = simplex.vertices.len() - 1; // k is the dimension of the simplex

        if k == 0 {
            return Ok(1.0); // Volume of a point is 1 by convention
        }

        let num_vertices = k + 1;

        // The Cayley-Menger matrix is (k+2)x(k+2)
        let matrix_dim = k + 2;
        let mut cm_matrix_data = vec![0.0; matrix_dim * matrix_dim];

        // Get the squared edge lengths for the simplex
        let squared_lengths = self.get_simplex_edge_lengths_squared(simplex)?;

        // Fill the Cayley-Menger matrix
        // Top row and left column are 1s, except for (0,0) which is 0.
        for i in 1..matrix_dim {
            cm_matrix_data[i] = 1.0; // First column
            cm_matrix_data[i * matrix_dim] = 1.0; // First row
        }

        // Fill the rest of the matrix with squared distances
        for i in 0..num_vertices {
            for j in i..num_vertices {
                let dist_sq = if i == j {
                    0.0
                } else {
                    // Find the squared length for edge (v_i, v_j)
                    // The squared_lengths map is keyed by a sorted pair of vertex indices.
                    let key = if simplex.vertices[i] < simplex.vertices[j] {
                        (simplex.vertices[i], simplex.vertices[j])
                    } else {
                        (simplex.vertices[j], simplex.vertices[i])
                    };
                    *squared_lengths.get(&key).ok_or_else(|| {
                        TopologyError::ManifoldError(format!("Missing edge length for {:?}", key))
                    })?
                };
                // Matrix indices are +1 because of the border
                cm_matrix_data[(i + 1) * matrix_dim + (j + 1)] = dist_sq;
                cm_matrix_data[(j + 1) * matrix_dim + (i + 1)] = dist_sq;
            }
        }

        let cm_tensor = CausalTensor::new(cm_matrix_data, vec![matrix_dim, matrix_dim])?;
        let det = determinant(&cm_tensor).map_err(|e| TopologyError::TensorError(e.to_string()))?;

        // Formula for squared k-volume
        let k_fac = (1..=k).map(|i| i as f64).product::<f64>();
        let denominator = 2.0_f64.powi(k as i32) * k_fac.powi(2);
        let sign = if k.is_multiple_of(2) { -1.0 } else { 1.0 }; // (-1)^(k+1)

        let vol_sq = (sign / denominator) * det;

        // Due to floating point inaccuracies, result can be a tiny negative number.
        if vol_sq < 0.0 { Ok(0.0) } else { Ok(vol_sq) }
    }

    /// Helper to get all edge lengths for a given simplex
    fn get_simplex_edge_lengths_squared(
        &self,
        simplex: &Simplex,
    ) -> Result<std::collections::HashMap<(usize, usize), f64>, TopologyError> {
        let metric = self
            .metric
            .as_ref()
            .ok_or(TopologyError::ManifoldError("Metric not found".into()))?;

        let skeleton_1 = self
            .complex
            .skeletons
            .get(1)
            .ok_or(TopologyError::DimensionMismatch(
                "1-skeleton not found".into(),
            ))?;

        let mut edge_lengths = std::collections::HashMap::new();

        let vertices = &simplex.vertices;
        for i in 0..vertices.len() {
            for j in (i + 1)..vertices.len() {
                let v1 = vertices[i];
                let v2 = vertices[j];

                // Create a temporary simplex for the edge to find its index
                let edge_simplex = Simplex::new(vec![v1, v2]);

                if let Some(edge_index) = skeleton_1.get_index(&edge_simplex) {
                    let length = metric.edge_lengths.get(&[edge_index]).ok_or(
                        TopologyError::IndexOutOfBounds("Edge length not found".into()),
                    )?;
                    edge_lengths.insert((v1, v2), length.powi(2));
                } else {
                    return Err(TopologyError::SimplexNotFound);
                }
            }
        }

        Ok(edge_lengths)
    }
}

// Helper function to calculate the determinant of a square matrix represented by CausalTensor.
// Using Laplace expansion, which is simple to implement but inefficient for large matrices.
// Given the low dimensionality of simplices in typical physics simulations, this is acceptable.
fn determinant(matrix: &CausalTensor<f64>) -> Result<f64, CausalTensorError> {
    let shape = matrix.shape();
    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(CausalTensorError::InvalidParameter(
            "Determinant requires a square matrix".into(),
        ));
    }
    let n = shape[0];

    if n == 0 {
        return Ok(1.0); // Determinant of a 0x0 matrix is 1
    }
    if n == 1 {
        return Ok(matrix.as_slice()[0]);
    }
    if n == 2 {
        let m = matrix.as_slice();
        return Ok(m[0] * m[3] - m[1] * m[2]);
    }

    let mut det = 0.0;
    for j1 in 0..n {
        let sign = if j1 % 2 == 0 { 1.0 } else { -1.0 };

        // Create sub-matrix (minor)
        let mut sub_matrix_data = Vec::with_capacity((n - 1) * (n - 1));
        for i in 1..n {
            for j in 0..n {
                if j == j1 {
                    continue;
                }
                sub_matrix_data.push(*matrix.get(&[i, j]).unwrap());
            }
        }
        let sub_matrix = CausalTensor::new(sub_matrix_data, vec![n - 1, n - 1])?;
        det += sign * matrix.get(&[0, j1]).unwrap() * determinant(&sub_matrix)?;
    }

    Ok(det)
}
