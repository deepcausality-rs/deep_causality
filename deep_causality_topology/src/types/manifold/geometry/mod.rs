/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CPU implementation of geometry operations for Manifold.
//!
//! Computes purely from the data-precision `D`: edge lengths in the metric are typed
//! `ReggeGeometry<D>` and volumes/determinants return `D`. The coordinate type `C` is
//! treated as an opaque identifier and is no longer bounded numerically here.

use deep_causality_algebra::RealField;
use deep_causality_linear::{DenseMatrix, determinant};
use deep_causality_num::FromPrimitive;

use crate::{Manifold, Simplex, SimplicialComplex, TopologyError};
use std::collections::HashMap;

impl<C, D> Manifold<SimplicialComplex<C>, D>
where
    C: RealField + FromPrimitive,
{
    /// CPU implementation of simplex volume squared calculation.
    ///
    /// Returns the volume in the **metric precision** `C` — Cayley-Menger inputs are
    /// edge lengths from the metric, and the volume lives in the same field. The
    /// manifold's data precision `D` does not participate in this computation; callers
    /// who need the volume in `D` precision are responsible for the conversion.
    pub(crate) fn simplex_volume_squared_impl(
        &self,
        simplex: &Simplex,
    ) -> Result<C, TopologyError> {
        let k = simplex.vertices.len() - 1;

        if k == 0 {
            return Ok(C::one());
        }

        let num_vertices = k + 1;
        let matrix_dim = k + 2;
        let mut cm_matrix_data = vec![C::zero(); matrix_dim * matrix_dim];

        let squared_lengths = self.get_simplex_edge_lengths_squared_impl(simplex)?;

        let one = C::one();
        for i in 1..matrix_dim {
            cm_matrix_data[i] = one;
            cm_matrix_data[i * matrix_dim] = one;
        }

        for i in 0..num_vertices {
            for j in i..num_vertices {
                let dist_sq = if i == j {
                    C::zero()
                } else {
                    let key = if simplex.vertices[i] < simplex.vertices[j] {
                        (simplex.vertices[i], simplex.vertices[j])
                    } else {
                        (simplex.vertices[j], simplex.vertices[i])
                    };
                    *squared_lengths.get(&key).ok_or_else(|| {
                        TopologyError::ManifoldError(format!("Missing edge length for {:?}", key))
                    })?
                };
                cm_matrix_data[(i + 1) * matrix_dim + (j + 1)] = dist_sq;
                cm_matrix_data[(j + 1) * matrix_dim + (i + 1)] = dist_sq;
            }
        }

        // `cm_matrix_data` is already row-major, so this is the same buffer read as a matrix
        // rather than as a rank-2 tensor. Its `(0,0)` entry is zero — the loop above writes `one`
        // only into indices `1..matrix_dim` — which is why the shared determinant's pivot search
        // is load-bearing here and not a refinement.
        let cm = DenseMatrix::from_vec(cm_matrix_data, matrix_dim, matrix_dim)
            .map_err(TopologyError::from)?;
        let det = determinant(&cm).map_err(TopologyError::from)?;

        // Squared k-volume formula: vol² = (-1)^(k+1) / (2^k * (k!)^2) * det(CM)
        let mut k_fac = C::one();
        for i in 1..=k {
            k_fac *=
                <C as FromPrimitive>::from_usize(i).expect("usize is representable in RealField");
        }
        let two = <C as FromPrimitive>::from_f64(2.0).expect("2.0 is representable");
        let mut two_pow_k = C::one();
        for _ in 0..k {
            two_pow_k *= two;
        }
        let denominator = two_pow_k * k_fac * k_fac;
        let sign = if k.is_multiple_of(2) {
            -C::one()
        } else {
            C::one()
        };

        let vol_sq = (sign / denominator) * det;

        if vol_sq < C::zero() {
            Ok(C::zero())
        } else {
            Ok(vol_sq)
        }
    }

    /// CPU implementation: get all edge lengths squared for a given simplex.
    fn get_simplex_edge_lengths_squared_impl(
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

                let edge_simplex = Simplex::new(vec![v1, v2]);

                if let Some(edge_index) = skeleton_1.get_index(&edge_simplex) {
                    let length = metric.edge_lengths.get(&[edge_index]).ok_or(
                        TopologyError::IndexOutOfBounds("Edge length not found".to_string()),
                    )?;
                    edge_lengths.insert((v1, v2), (*length) * (*length));
                } else {
                    return Err(TopologyError::SimplexNotFound());
                }
            }
        }

        Ok(edge_lengths)
    }
}
