/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// GrOps Implementation for GR (GaugeField<Lorentz, f64, f64>)
// =============================================================================

use crate::theories::general_relativity::gr_lie_mapping::expand_lie_to_riemann;
use crate::theories::general_relativity::gr_utils;
use crate::{
    GR, GeodesicState, GrOps, PhysicsError, geodesic_integrator_kernel, parallel_transport_kernel,
    proper_time_kernel,
};
use deep_causality_haft::RiemannMap;
use deep_causality_metric::{EastCoastMetric, LorentzianMetric};
use deep_causality_num::{Field, Float};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::GaugeFieldWitness;
use deep_causality_topology::{
    CurvatureSymmetry, CurvatureTensor, CurvatureTensorVector, CurvatureTensorWitness, TensorVector,
};

impl<S> GrOps<S> for GR<S>
where
    S: Field + Float + Clone + From<f64> + Into<f64> + Copy,
{
    fn ricci_tensor(&self) -> Result<CausalTensor<S>, PhysicsError> {
        let riemann = self.field_strength();
        let dim = 4;

        // Use East Coast metric type info (structure only)
        let metric_sig = EastCoastMetric::minkowski_4d().into_metric();

        let ct = CurvatureTensor::<S, (), (), (), ()>::new(
            riemann.clone(),
            metric_sig,
            CurvatureSymmetry::Riemann,
            dim,
        );

        let ricci_data = ct.ricci_tensor();
        Ok(CausalTensor::from_vec(ricci_data, &[dim, dim]))
    }

    fn ricci_scalar(&self) -> Result<S, PhysicsError> {
        // Use CurvatureTensor for the complex Riemann->Ricci contraction
        let ricci = self.ricci_tensor()?;
        let ricci_data = ricci.as_slice();

        // Use the metric from the field for the scalar contraction
        let metric = self.metric_tensor();
        let dim = 4;

        // Full 4x4 Matrix Inversion
        let inv_metric = gr_utils::invert_4x4(metric)?;

        let mut scalar = S::zero();
        // R = g^μν R_μν
        for mu in 0..dim {
            for nu in 0..dim {
                // Flattened index [mu, nu]
                let idx = mu * dim + nu;
                let g_upper = inv_metric[idx];
                let r_lower = ricci_data.get(idx).copied().unwrap_or(S::zero());
                scalar = scalar + (g_upper * r_lower);
            }
        }

        Ok(scalar)
    }

    fn einstein_tensor(&self) -> Result<CausalTensor<S>, PhysicsError> {
        // Use CurvatureTensor from topology for the Einstein tensor calculation
        let riemann = self.field_strength();
        let dim = 4;
        let metric_sig = EastCoastMetric::minkowski_4d().into_metric();

        let ct = CurvatureTensor::<S, (), (), (), ()>::new(
            riemann.clone(),
            metric_sig,
            CurvatureSymmetry::Riemann,
            dim,
        );

        // Use topology's einstein_tensor method
        let einstein_data = ct.einstein_tensor();
        Ok(CausalTensor::from_vec(einstein_data, &[dim, dim]))
    }

    fn kretschmann_scalar(&self) -> Result<S, PhysicsError> {
        let lie_fs = self.field_strength();
        let dim = 4;
        let metric_sig = EastCoastMetric::minkowski_4d().into_metric();

        // Expand Lie-algebra storage [points, 4, 4, 6] to geometric [4, 4, 4, 4]
        let riemann = expand_lie_to_riemann(lie_fs)?;

        // Get Inverse Metric for index raising
        let metric = self.metric_tensor();
        let inv_g = gr_utils::invert_4x4(metric)?;

        // Create CurvatureTensor and use topology's kretschmann_scalar_with_metric
        let ct = CurvatureTensor::<S, (), (), (), ()>::new(
            riemann,
            metric_sig,
            CurvatureSymmetry::Riemann,
            dim,
        );

        // Use topology method with the precomputed inverse metric
        Ok(ct.kretschmann_scalar_with_metric(&inv_g))
    }

    fn geodesic_deviation(&self, velocity: &[S], separation: &[S]) -> Result<Vec<S>, PhysicsError> {
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
        let ct =
            CurvatureTensorVector::<S>::new(riemann, metric_sig, CurvatureSymmetry::Riemann, dim);

        // Use RiemannMap HKT trait via witness type
        // D^2 ξ / dτ^2 = R(u, ξ)u
        let result_vector = CurvatureTensorWitness::curvature(ct, u, v, u_w);
        Ok(result_vector.into())
    }

    fn solve_geodesic(
        &self,
        initial_position: &[S],
        initial_velocity: &[S],
        proper_time_step: S,
        num_steps: usize,
    ) -> Result<Vec<GeodesicState<S>>, PhysicsError> {
        geodesic_integrator_kernel(
            initial_position,
            initial_velocity,
            self.connection(),
            proper_time_step,
            num_steps,
        )
    }

    fn proper_time(&self, path: &[Vec<S>]) -> Result<S, PhysicsError> {
        proper_time_kernel(path, self.metric_tensor())
    }

    fn parallel_transport(
        &self,
        initial_vector: &[S],
        path: &[Vec<S>],
    ) -> Result<Vec<S>, PhysicsError> {
        parallel_transport_kernel(initial_vector, path, self.connection())
    }

    fn metric_tensor(&self) -> &CausalTensor<S> {
        self.connection()
    }

    fn compute_riemann_from_christoffel(&self) -> CausalTensor<S> {
        // The coupling constant for GR is effectively 1.0
        // (structure constants encode the non-abelian part)
        GaugeFieldWitness::compute_field_strength_non_abelian(self, S::one())
    }

    fn momentum_constraint_field(
        &self,
        extrinsic_curvature: &CausalTensor<S>,
        matter_momentum: Option<&CausalTensor<S>>,
    ) -> Result<CausalTensor<S>, PhysicsError> {
        // =========================================================================
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
        let extract_spatial_metric = |p: usize| -> [[S; 3]; 3] {
            let base = p * metric_stride;

            // If metric is 4x4 (stride=16), extract spatial components g_{i+1,j+1}
            if metric_stride >= 16 {
                let mut gamma = [[S::zero(); 3]; 3];
                for (i, row) in gamma.iter_mut().enumerate() {
                    for (j, val) in row.iter_mut().enumerate() {
                        let idx = base + (i + 1) * 4 + (j + 1);
                        *val = metric_4d.get(idx).copied().unwrap_or(if i == j {
                            S::one()
                        } else {
                            S::zero()
                        });
                    }
                }
                gamma
            } else {
                // Fallback: identity metric (flat space)
                [
                    [S::one(), S::zero(), S::zero()],
                    [S::zero(), S::one(), S::zero()],
                    [S::zero(), S::zero(), S::one()],
                ]
            }
        };

        // -------------------------------------------------------------------------
        // 3. COMPUTE COVARIANT DIVERGENCE USING MANIFOLD TOPOLOGY
        // -------------------------------------------------------------------------
        // D_j T^j_i = ∂_j T^j_i + Γ^j_jk T^k_i - Γ^k_ji T^j_k

        let base_manifold = self.base();
        let complex = base_manifold.complex();

        // Allocate result: M_i for each point
        let mut result = vec![S::zero(); num_points * 3];

        for p in 0..num_points {
            let k_offset = if is_batched { p * 9 } else { 0 };

            // Extract spatial metric and its inverse at this point
            let gamma = extract_spatial_metric(p);
            let gamma_inv = gr_utils::invert_3x3(gamma)?;

            // Compute trace K = γ^ij K_ij
            let mut k_trace = S::zero();
            for (i, row) in gamma_inv.iter().enumerate() {
                for (j, &g_inv_ij) in row.iter().enumerate() {
                    let k_ij = k_data
                        .get(k_offset + i * 3 + j)
                        .copied()
                        .unwrap_or(S::zero());
                    k_trace = k_trace + g_inv_ij * k_ij;
                }
            }

            // Compute mixed tensor K^j_i = γ^jk K_ki
            let mut k_mixed = [[S::zero(); 3]; 3];
            for (j, row) in k_mixed.iter_mut().enumerate() {
                for (i, val) in row.iter_mut().enumerate() {
                    for (k, &g_inv_jk) in gamma_inv[j].iter().enumerate() {
                        let k_ki = k_data
                            .get(k_offset + k * 3 + i)
                            .copied()
                            .unwrap_or(S::zero());
                        *val = *val + g_inv_jk * k_ki;
                    }
                }
            }

            // Compute T^j_i = K^j_i - δ^j_i K
            let mut t_tensor = [[S::zero(); 3]; 3];
            for j in 0..3 {
                for i in 0..3 {
                    let delta = if i == j { S::one() } else { S::zero() };
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
            let mut christoffel = [[[S::zero(); 3]; 3]; 3];

            if !neighbors.is_empty() {
                for n_idx in &neighbors {
                    let gamma_n = extract_spatial_metric(*n_idx);
                    let weight = S::one() / <S as From<f64>>::from(neighbors.len() as f64);
                    let half = <S as From<f64>>::from(0.5);

                    for k in 0..3 {
                        for i in 0..3 {
                            for j in 0..3 {
                                for l in 0..3 {
                                    let d_gamma_jl = (gamma_n[j][l] - gamma[j][l]) * weight;
                                    let d_gamma_il = (gamma_n[i][l] - gamma[i][l]) * weight;
                                    let d_gamma_ij = (gamma_n[i][j] - gamma[i][j]) * weight;

                                    christoffel[k][i][j] = christoffel[k][i][j]
                                        + half
                                            * gamma_inv[k][l]
                                            * (d_gamma_jl + d_gamma_il - d_gamma_ij);
                                }
                            }
                        }
                    }
                }
            }

            // Compute ∂_j T^j_i using finite differences with neighbors
            let mut partial_div = [S::zero(); 3];

            if !neighbors.is_empty() {
                for n_idx in &neighbors {
                    let n_k_offset = if is_batched { n_idx * 9 } else { 0 };
                    let gamma_n = extract_spatial_metric(*n_idx);
                    let gamma_n_inv = gr_utils::invert_3x3(gamma_n)?;

                    // Compute T^j_i at neighbor
                    let mut k_trace_n = S::zero();
                    for (i, row) in gamma_n_inv.iter().enumerate() {
                        for (j, &g_n_inv_ij) in row.iter().enumerate() {
                            let k_ij = k_data
                                .get(n_k_offset + i * 3 + j)
                                .copied()
                                .unwrap_or(S::zero());
                            k_trace_n = k_trace_n + g_n_inv_ij * k_ij;
                        }
                    }

                    let mut k_mixed_n = [[S::zero(); 3]; 3];
                    for (j, row) in k_mixed_n.iter_mut().enumerate() {
                        for (i, val) in row.iter_mut().enumerate() {
                            for (k, &g_n_inv_jk) in gamma_n_inv[j].iter().enumerate() {
                                let k_ki = k_data
                                    .get(n_k_offset + k * 3 + i)
                                    .copied()
                                    .unwrap_or(S::zero());
                                *val = *val + g_n_inv_jk * k_ki;
                            }
                        }
                    }

                    let mut t_n = [[S::zero(); 3]; 3];
                    for j in 0..3 {
                        for i in 0..3 {
                            let delta = if i == j { S::one() } else { S::zero() };
                            t_n[j][i] = k_mixed_n[j][i] - delta * k_trace_n;
                        }
                    }

                    let weight = S::one() / <S as From<f64>>::from(neighbors.len() as f64);
                    for i in 0..3 {
                        for j in 0..3 {
                            partial_div[i] = partial_div[i] + (t_n[j][i] - t_tensor[j][i]) * weight;
                        }
                    }
                }
            }

            // Compute connection terms: Γ^j_jk T^k_i - Γ^k_ji T^j_k
            let mut connection_term = [S::zero(); 3];
            for i in 0..3 {
                for j in 0..3 {
                    for k in 0..3 {
                        connection_term[i] =
                            connection_term[i] + christoffel[j][j][k] * t_tensor[k][i];
                        connection_term[i] =
                            connection_term[i] - christoffel[k][j][i] * t_tensor[j][k];
                    }
                }
            }

            // -------------------------------------------------------------------------
            // 4. ASSEMBLE MOMENTUM CONSTRAINT
            // -------------------------------------------------------------------------
            let eight_pi = <S as From<f64>>::from(8.0 * std::f64::consts::PI);

            for i in 0..3 {
                let j_i = match matter_momentum {
                    Some(j) => j.as_slice().get(p * 3 + i).copied().unwrap_or(S::zero()),
                    None => S::zero(),
                };

                result[p * 3 + i] = partial_div[i] + connection_term[i] - eight_pi * j_i;
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
