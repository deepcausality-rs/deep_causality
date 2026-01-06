/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// GrOps Implementation for GR (GaugeField<Lorentz, f64, f64>)
// =============================================================================

use crate::theories::general_relativity::gr_utils;
use crate::{
    GR, GeodesicState, GrOps, PhysicsError, einstein_tensor_kernel, geodesic_integrator_kernel,
    parallel_transport_kernel, proper_time_kernel,
};
use deep_causality_haft::RiemannMap;
use deep_causality_metric::{EastCoastMetric, LorentzianMetric};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    CurvatureSymmetry, CurvatureTensor, CurvatureTensorVector, CurvatureTensorWitness, TensorVector,
};

impl GrOps for GR {
    fn ricci_tensor(&self) -> Result<CausalTensor<f64>, PhysicsError> {
        let riemann = self.field_strength();
        let dim = 4;

        // Use East Coast metric type info for the wrapper (needed for structure, though unused for Ricci contraction)
        let metric_sig = EastCoastMetric::minkowski_4d().into_metric();

        let ct = CurvatureTensor::<(), (), (), ()>::new(
            riemann.clone(),
            metric_sig,
            CurvatureSymmetry::Riemann,
            dim,
        );

        let ricci_data = ct.ricci_tensor();
        Ok(CausalTensor::from_vec(ricci_data, &[dim, dim]))
    }

    fn ricci_scalar(&self) -> Result<f64, PhysicsError> {
        // Use CurvatureTensor for the complex Riemann->Ricci contraction
        let ricci = self.ricci_tensor()?;
        let ricci_data = ricci.as_slice();

        // Use the metric from the field for the scalar contraction
        let metric = self.metric_tensor();
        let dim = 4;

        // Full 4x4 Matrix Inversion
        let inv_metric = gr_utils::invert_4x4(metric)?;

        let mut scalar = 0.0;
        // R = g^μν R_μν
        for mu in 0..dim {
            for nu in 0..dim {
                // Flattened index [mu, nu]
                let idx = mu * dim + nu;
                let g_upper = inv_metric[idx];
                let r_lower = ricci_data.get(idx).copied().unwrap_or(0.0);
                scalar += g_upper * r_lower;
            }
        }

        Ok(scalar)
    }

    fn einstein_tensor(&self) -> Result<CausalTensor<f64>, PhysicsError> {
        let ricci = self.ricci_tensor()?;
        let scalar = self.ricci_scalar()?;
        einstein_tensor_kernel(&ricci, scalar, self.metric_tensor())
    }

    fn kretschmann_scalar(&self) -> Result<f64, PhysicsError> {
        use crate::theories::general_relativity::gr_lie_mapping::expand_lie_to_riemann;

        let lie_fs = self.field_strength();
        let dim = 4;

        // Expand Lie-algebra storage [points, 4, 4, 6] to geometric [4, 4, 4, 4]
        let riemann = expand_lie_to_riemann(lie_fs)?;
        let r_data = riemann.as_slice();

        // Get Inverse Metric for index raising
        let metric = self.metric_tensor();
        let inv_g = gr_utils::invert_4x4(metric)?;

        // Helper to index flat 4D array
        let idx4 = |a, b, c, d| ((a * dim + b) * dim + c) * dim + d;

        // Storage for intermediate raised tensors
        let mut r_raised = r_data.to_vec(); // Start with R_abcd
        let mut temp = vec![0.0; dim * dim * dim * dim];

        // Raise 1st index: R^a_bcd = g^am R_mbcd
        for a in 0..dim {
            for b in 0..dim {
                for c in 0..dim {
                    for d in 0..dim {
                        let mut sum = 0.0;
                        for m in 0..dim {
                            sum += inv_g[a * dim + m] * r_raised[idx4(m, b, c, d)];
                        }
                        temp[idx4(a, b, c, d)] = sum;
                    }
                }
            }
        }
        r_raised.copy_from_slice(&temp);

        // Raise 2nd index: R^ab_cd = g^bn R^a_ncd
        for a in 0..dim {
            for b in 0..dim {
                for c in 0..dim {
                    for d in 0..dim {
                        let mut sum = 0.0;
                        for n in 0..dim {
                            sum += inv_g[b * dim + n] * r_raised[idx4(a, n, c, d)];
                        }
                        temp[idx4(a, b, c, d)] = sum;
                    }
                }
            }
        }
        r_raised.copy_from_slice(&temp);

        // Raise 3rd index: R^abc_d = g^cr R^ab_rd
        for a in 0..dim {
            for b in 0..dim {
                for c in 0..dim {
                    for d in 0..dim {
                        let mut sum = 0.0;
                        for r in 0..dim {
                            sum += inv_g[c * dim + r] * r_raised[idx4(a, b, r, d)];
                        }
                        temp[idx4(a, b, c, d)] = sum;
                    }
                }
            }
        }
        r_raised.copy_from_slice(&temp);

        // Raise 4th index: R^abcd = g^ds R^abc_s
        for a in 0..dim {
            for b in 0..dim {
                for c in 0..dim {
                    for d in 0..dim {
                        let mut sum = 0.0;
                        for s in 0..dim {
                            sum += inv_g[d * dim + s] * r_raised[idx4(a, b, c, s)];
                        }
                        temp[idx4(a, b, c, d)] = sum;
                    }
                }
            }
        }
        r_raised.copy_from_slice(&temp); // Now r_raised holds R^abcd

        // Final Contraction: K = R_abcd * R^abcd
        let mut k = 0.0;
        for i in 0..r_data.len() {
            k += r_data[i] * r_raised[i];
        }

        Ok(k)
    }

    fn geodesic_deviation(
        &self,
        velocity: &[f64],
        separation: &[f64],
    ) -> Result<Vec<f64>, PhysicsError> {
        use crate::theories::general_relativity::gr_lie_mapping::expand_lie_to_riemann;

        let lie_fs = self.field_strength();
        let dim = 4;
        let metric_sig = EastCoastMetric::minkowski_4d().into_metric();

        // Expand Lie storage to geometric for CurvatureTensor
        let riemann = expand_lie_to_riemann(lie_fs)?;

        // Use TensorVector for HKT safety contract
        let u = TensorVector::new(velocity);
        let v = TensorVector::new(separation);
        let u_w = u.clone();

        // Construct CurvatureTensorVector for HKT witness with geometric Riemann
        let ct = CurvatureTensorVector::new(riemann, metric_sig, CurvatureSymmetry::Riemann, dim);

        // Use RiemannMap HKT trait via witness type
        // D^2 ξ / dτ^2 = R(u, ξ)u
        let result_vector = CurvatureTensorWitness::curvature(ct, u, v, u_w);
        Ok(result_vector.into())
    }

    fn solve_geodesic(
        &self,
        initial_position: &[f64],
        initial_velocity: &[f64],
        proper_time_step: f64,
        num_steps: usize,
    ) -> Result<Vec<GeodesicState>, PhysicsError> {
        geodesic_integrator_kernel(
            initial_position,
            initial_velocity,
            self.connection(),
            proper_time_step,
            num_steps,
        )
    }

    fn proper_time(&self, path: &[Vec<f64>]) -> Result<f64, PhysicsError> {
        proper_time_kernel(path, self.metric_tensor())
    }

    fn parallel_transport(
        &self,
        initial_vector: &[f64],
        path: &[Vec<f64>],
    ) -> Result<Vec<f64>, PhysicsError> {
        parallel_transport_kernel(initial_vector, path, self.connection())
    }

    /// Returns the spacetime metric g_μν.
    ///
    /// # Implementation Note
    ///
    /// In the GR implementation, the metric tensor is stored in the `GaugeField`'s
    /// `connection` slot. This is a semantic overload: for particle physics gauge
    /// theories, connection = gauge potential (A_μ), but for GR, we store the metric
    /// here and compute Christoffel symbols on-the-fly when needed.
    ///
    /// The `field_strength` slot contains the Riemann curvature tensor in Lie-algebra
    /// representation `[points, 4, 4, 6]`, which is expanded to geometric form
    /// `[4, 4, 4, 4]` using `expand_lie_to_riemann()` when needed.
    fn metric_tensor(&self) -> &CausalTensor<f64> {
        self.connection()
    }

    fn compute_riemann_from_christoffel(&self) -> CausalTensor<f64> {
        use deep_causality_topology::GaugeFieldWitness;

        // The coupling constant for GR is effectively 1.0
        // (structure constants encode the non-abelian part)
        GaugeFieldWitness::compute_field_strength_non_abelian(self, 1.0)
    }

    fn momentum_constraint_field(
        &self,
        extrinsic_curvature: &CausalTensor<f64>,
        matter_momentum: Option<&CausalTensor<f64>>,
    ) -> Result<CausalTensor<f64>, PhysicsError> {
        // =========================================================================
        // PRODUCTION-GRADE IMPLEMENTATION
        // ADM Momentum Constraint: M_i = D_j(K^j_i - δ^j_i K) - 8πj_i
        // =========================================================================

        // -------------------------------------------------------------------------
        // 1. INPUT VALIDATION
        // -------------------------------------------------------------------------
        let k_shape = extrinsic_curvature.shape();

        // Validate K_ij tensor shape: [N, 3, 3] or [3, 3]
        let (num_points, is_batched) = match k_shape.len() {
            2 => {
                if k_shape[0] != 3 || k_shape[1] != 3 {
                    return Err(PhysicsError::DimensionMismatch(format!(
                        "Expected K_ij shape [3, 3], got {:?}",
                        k_shape
                    )));
                }
                (1, false)
            }
            3 => {
                if k_shape[1] != 3 || k_shape[2] != 3 {
                    return Err(PhysicsError::DimensionMismatch(format!(
                        "Expected K_ij shape [N, 3, 3], got {:?}",
                        k_shape
                    )));
                }
                (k_shape[0], true)
            }
            _ => {
                return Err(PhysicsError::DimensionMismatch(format!(
                    "K_ij must be 2D [3,3] or 3D [N,3,3], got {:?}",
                    k_shape
                )));
            }
        };

        // Validate matter momentum if provided
        if let Some(j) = matter_momentum {
            let expected_size = num_points * 3;
            if j.as_slice().len() != expected_size {
                return Err(PhysicsError::DimensionMismatch(format!(
                    "Matter momentum j_i size mismatch: expected {}, got {}",
                    expected_size,
                    j.as_slice().len()
                )));
            }
        }

        let k_data = extrinsic_curvature.as_slice();

        // -------------------------------------------------------------------------
        // 2. EXTRACT SPATIAL 3-METRIC FROM 4D METRIC
        // -------------------------------------------------------------------------
        // The 4D metric g_μν is stored in self.connection() (semantic overload).
        // In ADM form: ds² = -α²dt² + γ_ij(dx^i + β^i dt)(dx^j + β^j dt)
        // For the spatial slice: γ_ij = g_ij (i,j ∈ {1,2,3} → indices 1,2,3 of 4D metric)

        let metric_4d = self.connection().as_slice();
        let metric_shape = self.connection().shape();

        // Determine metric stride based on storage format
        let metric_stride = if metric_shape.len() >= 2 {
            metric_shape[metric_shape.len() - 2] * metric_shape[metric_shape.len() - 1]
        } else {
            16 // Fallback: 4x4 metric
        };

        // Helper: Extract γ_ij (3x3 spatial metric) from g_μν
        let extract_spatial_metric = |p: usize| -> [[f64; 3]; 3] {
            let base = p * metric_stride;

            // If metric is 4x4 (stride=16), extract spatial components g_{i+1,j+1}
            if metric_stride >= 16 {
                let mut gamma = [[0.0; 3]; 3];
                for (i, row) in gamma.iter_mut().enumerate() {
                    for (j, val) in row.iter_mut().enumerate() {
                        let idx = base + (i + 1) * 4 + (j + 1);
                        *val =
                            metric_4d
                                .get(idx)
                                .copied()
                                .unwrap_or(if i == j { 1.0 } else { 0.0 });
                    }
                }
                gamma
            } else {
                // Fallback: identity metric (flat space)
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            }
        };

        // Helper: Compute inverse of 3x3 matrix
        let invert_3x3 = |m: [[f64; 3]; 3]| -> Result<[[f64; 3]; 3], PhysicsError> {
            let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
                - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
                + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);

            if det.abs() < 1e-14 {
                return Err(PhysicsError::NumericalInstability(format!(
                    "Singular spatial metric (det = {:.2e})",
                    det
                )));
            }

            let inv = 1.0 / det;
            Ok([
                [
                    inv * (m[1][1] * m[2][2] - m[1][2] * m[2][1]),
                    inv * (m[0][2] * m[2][1] - m[0][1] * m[2][2]),
                    inv * (m[0][1] * m[1][2] - m[0][2] * m[1][1]),
                ],
                [
                    inv * (m[1][2] * m[2][0] - m[1][0] * m[2][2]),
                    inv * (m[0][0] * m[2][2] - m[0][2] * m[2][0]),
                    inv * (m[0][2] * m[1][0] - m[0][0] * m[1][2]),
                ],
                [
                    inv * (m[1][0] * m[2][1] - m[1][1] * m[2][0]),
                    inv * (m[0][1] * m[2][0] - m[0][0] * m[2][1]),
                    inv * (m[0][0] * m[1][1] - m[0][1] * m[1][0]),
                ],
            ])
        };

        // -------------------------------------------------------------------------
        // 3. COMPUTE COVARIANT DIVERGENCE USING MANIFOLD TOPOLOGY
        // -------------------------------------------------------------------------
        // D_j T^j_i = ∂_j T^j_i + Γ^j_jk T^k_i - Γ^k_ji T^j_k

        let base_manifold = self.base();
        let complex = base_manifold.complex();

        // Allocate result: M_i for each point
        let mut result = vec![0.0; num_points * 3];

        for p in 0..num_points {
            let k_offset = if is_batched { p * 9 } else { 0 };

            // Extract spatial metric and its inverse at this point
            let gamma = extract_spatial_metric(p);
            let gamma_inv = invert_3x3(gamma)?;

            // Compute trace K = γ^ij K_ij
            let mut k_trace = 0.0;
            for (i, row) in gamma_inv.iter().enumerate() {
                for (j, &g_inv_ij) in row.iter().enumerate() {
                    let k_ij = k_data.get(k_offset + i * 3 + j).copied().unwrap_or(0.0);
                    k_trace += g_inv_ij * k_ij;
                }
            }

            // Compute mixed tensor K^j_i = γ^jk K_ki
            let mut k_mixed = [[0.0; 3]; 3];
            for (j, row) in k_mixed.iter_mut().enumerate() {
                for (i, val) in row.iter_mut().enumerate() {
                    for (k, &g_inv_jk) in gamma_inv[j].iter().enumerate() {
                        let k_ki = k_data.get(k_offset + k * 3 + i).copied().unwrap_or(0.0);
                        *val += g_inv_jk * k_ki;
                    }
                }
            }

            // Compute T^j_i = K^j_i - δ^j_i K
            let mut t_tensor = [[0.0; 3]; 3];
            for j in 0..3 {
                for i in 0..3 {
                    let delta = if i == j { 1.0 } else { 0.0 };
                    t_tensor[j][i] = k_mixed[j][i] - delta * k_trace;
                }
            }

            // Get neighbor indices from manifold topology
            let neighbors: Vec<usize> = complex.skeletons()[0]
                .simplices()
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != p && *idx < num_points)
                .take(6)
                .map(|(idx, _)| idx)
                .collect();

            // Compute spatial Christoffel symbols Γ^k_ij from metric derivatives
            let mut christoffel = [[[0.0; 3]; 3]; 3];

            if !neighbors.is_empty() {
                for n_idx in &neighbors {
                    let gamma_n = extract_spatial_metric(*n_idx);
                    let weight = 1.0 / (neighbors.len() as f64);

                    for k in 0..3 {
                        for i in 0..3 {
                            for j in 0..3 {
                                for l in 0..3 {
                                    let d_gamma_jl = (gamma_n[j][l] - gamma[j][l]) * weight;
                                    let d_gamma_il = (gamma_n[i][l] - gamma[i][l]) * weight;
                                    let d_gamma_ij = (gamma_n[i][j] - gamma[i][j]) * weight;

                                    christoffel[k][i][j] += 0.5
                                        * gamma_inv[k][l]
                                        * (d_gamma_jl + d_gamma_il - d_gamma_ij);
                                }
                            }
                        }
                    }
                }
            }

            // Compute ∂_j T^j_i using finite differences with neighbors
            let mut partial_div = [0.0; 3];

            if !neighbors.is_empty() {
                for n_idx in &neighbors {
                    let n_k_offset = if is_batched { n_idx * 9 } else { 0 };
                    let gamma_n = extract_spatial_metric(*n_idx);
                    let gamma_n_inv = invert_3x3(gamma_n)?;

                    // Compute T^j_i at neighbor
                    let mut k_trace_n = 0.0;
                    for (i, row) in gamma_n_inv.iter().enumerate() {
                        for (j, &g_n_inv_ij) in row.iter().enumerate() {
                            let k_ij = k_data.get(n_k_offset + i * 3 + j).copied().unwrap_or(0.0);
                            k_trace_n += g_n_inv_ij * k_ij;
                        }
                    }

                    let mut k_mixed_n = [[0.0; 3]; 3];
                    for (j, row) in k_mixed_n.iter_mut().enumerate() {
                        for (i, val) in row.iter_mut().enumerate() {
                            for (k, &g_n_inv_jk) in gamma_n_inv[j].iter().enumerate() {
                                let k_ki =
                                    k_data.get(n_k_offset + k * 3 + i).copied().unwrap_or(0.0);
                                *val += g_n_inv_jk * k_ki;
                            }
                        }
                    }

                    let mut t_n = [[0.0; 3]; 3];
                    for j in 0..3 {
                        for i in 0..3 {
                            let delta = if i == j { 1.0 } else { 0.0 };
                            t_n[j][i] = k_mixed_n[j][i] - delta * k_trace_n;
                        }
                    }

                    let weight = 1.0 / (neighbors.len() as f64);
                    for i in 0..3 {
                        for j in 0..3 {
                            partial_div[i] += (t_n[j][i] - t_tensor[j][i]) * weight;
                        }
                    }
                }
            }

            // Compute connection terms: Γ^j_jk T^k_i - Γ^k_ji T^j_k
            let mut connection_term = [0.0; 3];
            for i in 0..3 {
                for j in 0..3 {
                    for k in 0..3 {
                        connection_term[i] += christoffel[j][j][k] * t_tensor[k][i];
                        connection_term[i] -= christoffel[k][j][i] * t_tensor[j][k];
                    }
                }
            }

            // -------------------------------------------------------------------------
            // 4. ASSEMBLE MOMENTUM CONSTRAINT
            // -------------------------------------------------------------------------
            for i in 0..3 {
                let j_i = match matter_momentum {
                    Some(j) => j.as_slice().get(p * 3 + i).copied().unwrap_or(0.0),
                    None => 0.0,
                };

                result[p * 3 + i] =
                    partial_div[i] + connection_term[i] - 8.0 * std::f64::consts::PI * j_i;
            }
        }

        let output_shape = if is_batched {
            vec![num_points, 3]
        } else {
            vec![3]
        };
        Ok(CausalTensor::from_vec(result, &output_shape))
    }
}
