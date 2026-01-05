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

        // Map slice to 3x3 for easier indexing: g_ij = g[i*3 + j]
        let g00 = g[0];
        let g01 = g[1];
        let g02 = g[2];
        let g10 = g[3];
        let g11 = g[4];
        let g12 = g[5];
        let g20 = g[6];
        let g21 = g[7];
        let g22 = g[8];

        // Determinant by cofactor expansion
        // det = 00(11*22 - 12*21) - 01(10*22 - 12*20) + 02(10*21 - 11*20)
        let det = g00 * (g11 * g22 - g12 * g21) - g01 * (g10 * g22 - g12 * g20)
            + g02 * (g10 * g21 - g11 * g20);

        if det.abs() < 1e-12 {
            return Err(PhysicsError::NumericalInstability(
                "Spatial metric determinant is zero".into(),
            ));
        }

        let inv_det = 1.0 / det;

        // Cofactor matrix elements (transposed)
        // M^00
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
        let k_tensor = self.extrinsic_curvature.as_slice(); // K_ij (3x3 row major)

        let mut k_mixed_trace = 0.0; // K = K^i_i = γ^ij K_ij
        let mut k_sq_contracted = 0.0; // K_ij K^ij = K_ij K_lm γ^il γ^jm

        // Calculate K^ij and contractions
        // First raise one index: K^i_k = γ^ij K_jk
        // Then trace: K = K^i_i
        // Also K_ij K^ij = K_ij (γ^il γ^jm K_lm) = K_ij K^ij

        // Let's compute K^ij = γ^ia γ^jb K_ab
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

        // Trace of K^i_j = γ^ik K_kj
        for i in 0..3 {
            for j in 0..3 {
                let k_ij = k_tensor[i * 3 + j];
                k_mixed_trace += inv_gamma[i][j] * k_ij;
            }
        }

        // Contraction K_ij K^ij
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

        // H = R + K^2 - K_ij K^ij - 16πρ
        let h = r + (k_mixed_trace * k_mixed_trace)
            - k_sq_contracted
            - 16.0 * std::f64::consts::PI * rho;

        Ok(CausalTensor::new(vec![h], vec![1]).unwrap())
    }

    fn momentum_constraint(
        &self,
        _matter_momentum: Option<&CausalTensor<f64>>,
    ) -> Result<CausalTensor<f64>, PhysicsError> {
        // M_i = D_j (K^j_i - γ^j_i K) - 8πj_i
        // Requires spatial derivatives of K and metric (Christoffel symbols of 3-metric).
        // Since AdmState is currently point-wise without neighbor info or pre-computed derivatives,
        // we cannot compute this constraint.
        Err(PhysicsError::CalculationError(
            "Momentum constraint requires spatial derivatives (Christoffel symbols) pending manifold integration".into()
        ))
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
