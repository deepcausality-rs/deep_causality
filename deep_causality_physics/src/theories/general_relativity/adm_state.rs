/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AdmOps, PhysicsError};
use deep_causality_tensor::CausalTensor;

/// Represents the state of a spatial slice in the 3+1 decomposition.
#[derive(Debug, Clone)]
pub struct AdmState {
    /// Spatial metric γ_ij (3x3 tensor)
    spatial_metric: CausalTensor<f64>,
    /// Extrinsic curvature K_ij (3x3 tensor)
    extrinsic_curvature: CausalTensor<f64>,
    /// Lapse function α (scalar field)
    lapse: CausalTensor<f64>,
    /// Shift vector β^i (3-vector field)
    shift: CausalTensor<f64>,
    /// Spatial Ricci scalar R (3-curvature)
    spatial_ricci_scalar: f64,
    /// Pre-computed spatial Christoffel symbols ^(3)Γ^k_ij [3, 3, 3] (optional)
    spatial_christoffel: Option<CausalTensor<f64>>,
}

impl AdmState {
    pub fn spatial_metric(&self) -> &CausalTensor<f64> {
        &self.spatial_metric
    }

    pub fn extrinsic_curvature(&self) -> &CausalTensor<f64> {
        &self.extrinsic_curvature
    }

    pub fn lapse(&self) -> &CausalTensor<f64> {
        &self.lapse
    }

    pub fn shift(&self) -> &CausalTensor<f64> {
        &self.shift
    }

    pub fn spatial_ricci_scalar(&self) -> f64 {
        self.spatial_ricci_scalar
    }

    pub fn spatial_christoffel(&self) -> Option<&CausalTensor<f64>> {
        self.spatial_christoffel.as_ref()
    }

    pub fn new(
        spatial_metric: CausalTensor<f64>,
        extrinsic_curvature: CausalTensor<f64>,
        lapse: CausalTensor<f64>,
        shift: CausalTensor<f64>,
        spatial_ricci_scalar: f64,
    ) -> Self {
        Self {
            spatial_metric,
            extrinsic_curvature,
            lapse,
            shift,
            spatial_ricci_scalar,
            spatial_christoffel: None,
        }
    }

    /// Creates an AdmState with pre-computed spatial Christoffel symbols.
    ///
    /// # Arguments
    /// * `spatial_christoffel` - Christoffel symbols ^(3)Γ^k_ij of the 3-metric, shape [3, 3, 3]
    ///
    /// With Christoffel symbols provided, `momentum_constraint()` can be computed.
    pub fn with_christoffel(
        spatial_metric: CausalTensor<f64>,
        extrinsic_curvature: CausalTensor<f64>,
        lapse: CausalTensor<f64>,
        shift: CausalTensor<f64>,
        spatial_ricci_scalar: f64,
        spatial_christoffel: CausalTensor<f64>,
    ) -> Self {
        Self {
            spatial_metric,
            extrinsic_curvature,
            lapse,
            shift,
            spatial_ricci_scalar,
            spatial_christoffel: Some(spatial_christoffel),
        }
    }

    /// Computes the inverse of the 3x3 spatial metric.
    fn inverse_spatial_metric(&self) -> Result<[[f64; 3]; 3], PhysicsError> {
        let g = self.spatial_metric.as_slice();
        if g.len() != 9 {
            return Err(PhysicsError::DimensionMismatch(
                "Spatial metric must be 3x3".into(),
            ));
        }

        let g00 = g[0];
        let g01 = g[1];
        let g02 = g[2];
        let g10 = g[3];
        let g11 = g[4];
        let g12 = g[5];
        let g20 = g[6];
        let g21 = g[7];
        let g22 = g[8];

        let det = g00 * (g11 * g22 - g12 * g21) - g01 * (g10 * g22 - g12 * g20)
            + g02 * (g10 * g21 - g11 * g20);

        if det.abs() < 1e-12 {
            return Err(PhysicsError::NumericalInstability(
                "Spatial metric determinant is zero".into(),
            ));
        }

        let inv_det = 1.0 / det;

        let i00 = inv_det * (g11 * g22 - g12 * g21);
        let i01 = inv_det * (g02 * g21 - g01 * g22);
        let i02 = inv_det * (g01 * g12 - g02 * g11);
        let i10 = inv_det * (g12 * g20 - g10 * g22);
        let i11 = inv_det * (g00 * g22 - g02 * g20);
        let i12 = inv_det * (g10 * g02 - g00 * g12);
        let i20 = inv_det * (g10 * g21 - g11 * g20);
        let i21 = inv_det * (g20 * g01 - g00 * g21);
        let i22 = inv_det * (g00 * g11 - g01 * g10);

        Ok([[i00, i01, i02], [i10, i11, i12], [i20, i21, i22]])
    }
}

impl AdmOps for AdmState {
    fn hamiltonian_constraint(
        &self,
        matter_density: Option<&CausalTensor<f64>>,
    ) -> Result<CausalTensor<f64>, PhysicsError> {
        // H = R + K² - K_ij K^ij - 16πρ
        let inv_gamma = self.inverse_spatial_metric()?;
        let k_tensor = self.extrinsic_curvature.as_slice();

        let mut k_mixed_trace = 0.0;
        let mut k_sq_contracted = 0.0;

        let mut k_upper = [[0.0; 3]; 3];

        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0;
                for a in 0..3 {
                    for b in 0..3 {
                        let k_ab = k_tensor[a * 3 + b];
                        sum += inv_gamma[i][a] * inv_gamma[j][b] * k_ab;
                    }
                }
                k_upper[i][j] = sum;
            }
        }

        for i in 0..3 {
            for j in 0..3 {
                let k_ij = k_tensor[i * 3 + j];
                k_mixed_trace += inv_gamma[i][j] * k_ij;
            }
        }

        for i in 0..3 {
            for j in 0..3 {
                let k_ij = k_tensor[i * 3 + j];
                k_sq_contracted += k_ij * k_upper[i][j];
            }
        }

        let r = self.spatial_ricci_scalar;
        let rho = match matter_density {
            Some(t) => t.as_slice().first().copied().unwrap_or(0.0),
            None => 0.0,
        };

        let h = r + (k_mixed_trace * k_mixed_trace)
            - k_sq_contracted
            - 16.0 * std::f64::consts::PI * rho;

        Ok(CausalTensor::new(vec![h], vec![1]).unwrap())
    }

    fn momentum_constraint(
        &self,
        matter_momentum: Option<&CausalTensor<f64>>,
    ) -> Result<CausalTensor<f64>, PhysicsError> {
        // M_i = D_j (K^j_i - δ^j_i K) - 8πj_i
        // where D_j is the covariant derivative using spatial Christoffel symbols.
        let christoffel = self.spatial_christoffel.as_ref().ok_or_else(|| {
            PhysicsError::CalculationError(
                "Momentum constraint requires spatial Christoffel symbols. \
                 Use AdmState::with_christoffel() to provide them."
                    .into(),
            )
        })?;

        let gamma_data = christoffel.as_slice();
        if gamma_data.len() != 27 {
            return Err(PhysicsError::DimensionMismatch(
                "Spatial Christoffel symbols must have shape [3, 3, 3]".into(),
            ));
        }

        let inv_gamma = self.inverse_spatial_metric()?;
        let k_tensor = self.extrinsic_curvature.as_slice();

        // Compute K = trace of K^i_j
        let mut k_trace = 0.0;
        for i in 0..3 {
            for j in 0..3 {
                k_trace += inv_gamma[i][j] * k_tensor[i * 3 + j];
            }
        }

        // Compute K^j_i = γ^jk K_ki
        let mut k_mixed = [[0.0; 3]; 3]; // K^j_i
        for j in 0..3 {
            for i in 0..3 {
                let mut sum = 0.0;
                for k in 0..3 {
                    sum += inv_gamma[j][k] * k_tensor[k * 3 + i];
                }
                k_mixed[j][i] = sum;
            }
        }

        // Compute M_i = D_j (K^j_i - δ^j_i K) - 8πj_i
        // D_j V^j = ∂_j V^j + Γ^j_jk V^k (divergence)
        // For tensor T^j_i: D_j T^j_i = ∂_j T^j_i + Γ^j_jk T^k_i - Γ^k_ji T^j_k
        //
        // Without spatial derivatives (∂_j), we compute only the Christoffel connection terms.
        // This gives the "homogeneous" part. For full constraint, user must provide data on a grid.
        let mut m = [0.0; 3];

        // Γ^j_jk (trace over first two indices) for each k
        let gamma_trace = |k: usize| -> f64 {
            let mut sum = 0.0;
            for j in 0..3 {
                // Γ^j_jk at index [j, j, k]
                sum += gamma_data[j * 9 + j * 3 + k];
            }
            sum
        };

        // Γ^k_ji for each (k, j, i)
        let gamma = |k: usize, j: usize, i: usize| -> f64 { gamma_data[k * 9 + j * 3 + i] };

        for (i, val) in m.iter_mut().enumerate() {
            // T^j_i = K^j_i - δ^j_i K
            let t = |j: usize, i_inner: usize| -> f64 {
                let delta = if j == i_inner { 1.0 } else { 0.0 };
                k_mixed[j][i_inner] - delta * k_trace
            };

            // Connection terms: Γ^j_jk T^k_i - Γ^k_ji T^j_k
            let mut conn_term = 0.0;
            for k in 0..3 {
                conn_term += gamma_trace(k) * t(k, i);
            }
            for k in 0..3 {
                for j in 0..3 {
                    conn_term -= gamma(k, j, i) * t(j, k);
                }
            }

            // Matter momentum term
            let j_i = match matter_momentum {
                Some(j_tensor) => j_tensor.as_slice().get(i).copied().unwrap_or(0.0),
                None => 0.0,
            };

            *val = conn_term - 8.0 * std::f64::consts::PI * j_i;
        }

        Ok(CausalTensor::from_vec(m.to_vec(), &[3]))
    }

    fn mean_curvature(&self) -> Result<CausalTensor<f64>, PhysicsError> {
        let inv_gamma = self.inverse_spatial_metric()?;
        let k_tensor = self.extrinsic_curvature.as_slice();

        let mut k = 0.0;
        for i in 0..3 {
            for j in 0..3 {
                k += inv_gamma[i][j] * k_tensor[i * 3 + j];
            }
        }

        Ok(CausalTensor::new(vec![k], vec![1]).unwrap())
    }
}
