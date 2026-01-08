/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CPU implementation of geometry operations for Manifold.

use deep_causality_num::{Float, Zero};
use std::iter::Product;

use crate::{Manifold, Simplex, TopologyError};
use deep_causality_tensor::{CausalTensor, CausalTensorError};
use std::collections::HashMap;

impl<C, D> Manifold<C, D>
where
    C: Float + Zero + Copy + PartialOrd + From<f64> + Into<f64> + Product,
{
    /// CPU implementation of simplex volume squared calculation.
    pub(crate) fn simplex_volume_squared_cpu(&self, simplex: &Simplex) -> Result<C, TopologyError> {
        let k = simplex.vertices.len() - 1; // k is the dimension of the simplex

        if k == 0 {
            return Ok(<C as From<f64>>::from(1.0)); // Volume of a point is 1 by convention
        }

        let num_vertices = k + 1;

        // The Cayley-Menger matrix is (k+2)x(k+2)
        let matrix_dim = k + 2;
        let mut cm_matrix_data = vec![C::zero(); matrix_dim * matrix_dim];

        // Get the squared edge lengths for the simplex
        let squared_lengths = self.get_simplex_edge_lengths_squared_cpu(simplex)?;

        // Fill the Cayley-Menger matrix
        // Top row and left column are 1s, except for (0,0) which is 0.
        for i in 1..matrix_dim {
            cm_matrix_data[i] = <C as From<f64>>::from(1.0); // First column
            cm_matrix_data[i * matrix_dim] = <C as From<f64>>::from(1.0); // First row
        }

        // Fill the rest of the matrix with squared distances
        for i in 0..num_vertices {
            for j in i..num_vertices {
                let dist_sq = if i == j {
                    C::zero()
                } else {
                    // Find the squared length for edge (v_i, v_j)
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
        let det =
            determinant_cpu(&cm_tensor).map_err(|e| TopologyError::TensorError(e.to_string()))?;

        // Formula for squared k-volume
        let k_fac = (1..=k)
            .map(|i| <C as From<f64>>::from(i as f64))
            .product::<C>();
        let denominator = <C as From<f64>>::from(2.0f64).powi(k as i32) * k_fac.powi(2);
        let sign = if k.is_multiple_of(2) {
            <C as From<f64>>::from(-1.0)
        } else {
            <C as From<f64>>::from(1.0)
        }; // (-1)^(k+1)

        let vol_sq = (sign / denominator) * det;

        // Due to floating point inaccuracies, result can be a tiny negative number.
        if vol_sq < C::zero() {
            Ok(C::zero())
        } else {
            Ok(vol_sq)
        }
    }

    /// CPU implementation: get all edge lengths for a given simplex.
    fn get_simplex_edge_lengths_squared_cpu(
        &self,
        simplex: &Simplex,
    ) -> Result<HashMap<(usize, usize), C>, TopologyError> {
        let metric = self
            .metric
            .as_ref()
            .ok_or(TopologyError::ManifoldError("Metric not found".to_string()))?;

        let skeleton_1 = self
            .complex
            .skeletons
            .get(1)
            .ok_or(TopologyError::DimensionMismatch(
                "1-skeleton not found".to_string(),
            ))?;

        let mut edge_lengths = HashMap::new();

        let vertices = &simplex.vertices;
        for i in 0..vertices.len() {
            for j in (i + 1)..vertices.len() {
                let v1 = vertices[i];
                let v2 = vertices[j];

                // Create a temporary simplex for the edge to find its index
                let edge_simplex = Simplex::new(vec![v1, v2]);

                if let Some(edge_index) = skeleton_1.get_index(&edge_simplex) {
                    let length = metric.edge_lengths.get(&[edge_index]).ok_or(
                        TopologyError::IndexOutOfBounds("Edge length not found".to_string()),
                    )?;
                    edge_lengths.insert((v1, v2), length.powi(2));
                } else {
                    return Err(TopologyError::SimplexNotFound());
                }
            }
        }

        Ok(edge_lengths)
    }
}

/// CPU implementation of determinant using Laplace expansion.
pub(crate) fn determinant_cpu<T>(matrix: &CausalTensor<T>) -> Result<T, CausalTensorError>
where
    T: Float + Zero + Copy + PartialOrd + From<f64> + Into<f64>,
{
    let shape = matrix.shape();
    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(CausalTensorError::InvalidParameter(
            "Determinant requires a square matrix".into(),
        ));
    }
    let n = shape[0];

    if n == 0 {
        return Ok(<T as From<f64>>::from(1.0)); // Determinant of a 0x0 matrix is 1
    }
    if n == 1 {
        return Ok(matrix.as_slice()[0]);
    }
    if n == 2 {
        let m = matrix.as_slice();
        return Ok(m[0] * m[3] - m[1] * m[2]);
    }

    let mut det = T::zero();
    for j1 in 0..n {
        let sign = if j1 % 2 == 0 {
            <T as From<f64>>::from(1.0)
        } else {
            <T as From<f64>>::from(-1.0)
        };

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
        det = det + sign * *matrix.get(&[0, j1]).unwrap() * determinant_cpu(&sub_matrix)?;
    }

    Ok(det)
}
