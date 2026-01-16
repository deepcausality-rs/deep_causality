/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AdmOps, PhysicsError};
use deep_causality_num::Field;
use deep_causality_tensor::CausalTensor;
use std::marker::PhantomData;

/// Represents the state of a spatial slice in the 3+1 decomposition.
///
/// # Type Parameters
/// * `S` - Scalar type (e.g., `f32`, `f64`, `DoubleFloat`)
#[derive(Debug, Clone)]
pub struct AdmState<S>
where
    S: Field + Clone + From<f64> + Into<f64>,
{
    /// Spatial metric γ_ij (3x3 tensor)
    spatial_metric: CausalTensor<S>,
    /// Extrinsic curvature K_ij (3x3 tensor)
    extrinsic_curvature: CausalTensor<S>,
    /// Lapse function α (scalar field)
    lapse: CausalTensor<S>,
    /// Shift vector β^i (3-vector field)
    shift: CausalTensor<S>,
    /// Spatial Ricci scalar R (3-curvature)
    spatial_ricci_scalar: S,
    /// Pre-computed spatial Christoffel symbols ^(3)Γ^k_ij [3, 3, 3] (optional)
    spatial_christoffel: Option<CausalTensor<S>>,
    /// Phantom data for type parameter
    _marker: PhantomData<S>,
}

impl<S> Default for AdmState<S>
where
    S: Field + Clone + From<f64> + Into<f64> + Default,
{
    fn default() -> Self {
        let zero = <S as From<f64>>::from(0.0);
        Self {
            spatial_metric: CausalTensor::zeros(&[3, 3]),
            extrinsic_curvature: CausalTensor::zeros(&[3, 3]),
            lapse: CausalTensor::from_vec(vec![<S as From<f64>>::from(1.0)], &[1]),
            shift: CausalTensor::zeros(&[3]),
            spatial_ricci_scalar: zero,
            spatial_christoffel: None,
            _marker: PhantomData,
        }
    }
}

impl<S> AdmState<S>
where
    S: Field + Clone + From<f64> + Into<f64> + Copy,
{
    pub fn spatial_metric(&self) -> &CausalTensor<S> {
        &self.spatial_metric
    }

    pub fn extrinsic_curvature(&self) -> &CausalTensor<S> {
        &self.extrinsic_curvature
    }

    pub fn lapse(&self) -> &CausalTensor<S> {
        &self.lapse
    }

    pub fn shift(&self) -> &CausalTensor<S> {
        &self.shift
    }

    pub fn spatial_ricci_scalar(&self) -> S {
        self.spatial_ricci_scalar
    }

    pub fn spatial_christoffel(&self) -> Option<&CausalTensor<S>> {
        self.spatial_christoffel.as_ref()
    }

    pub fn new(
        spatial_metric: CausalTensor<S>,
        extrinsic_curvature: CausalTensor<S>,
        lapse: CausalTensor<S>,
        shift: CausalTensor<S>,
        spatial_ricci_scalar: S,
    ) -> Self {
        Self {
            spatial_metric,
            extrinsic_curvature,
            lapse,
            shift,
            spatial_ricci_scalar,
            spatial_christoffel: None,
            _marker: PhantomData,
        }
    }

    /// Creates an AdmState with pre-computed spatial Christoffel symbols.
    ///
    /// # Arguments
    /// * `spatial_christoffel` - Christoffel symbols ^(3)Γ^k_ij of the 3-metric, shape [3, 3, 3]
    ///
    /// With Christoffel symbols provided, `momentum_constraint()` can be computed.
    pub fn with_christoffel(
        spatial_metric: CausalTensor<S>,
        extrinsic_curvature: CausalTensor<S>,
        lapse: CausalTensor<S>,
        shift: CausalTensor<S>,
        spatial_ricci_scalar: S,
        spatial_christoffel: CausalTensor<S>,
    ) -> Self {
        Self {
            spatial_metric,
            extrinsic_curvature,
            lapse,
            shift,
            spatial_ricci_scalar,
            spatial_christoffel: Some(spatial_christoffel),
            _marker: PhantomData,
        }
    }

    /// Computes the inverse of the 3x3 spatial metric.
    fn inverse_spatial_metric(&self) -> Result<[[S; 3]; 3], PhysicsError> {
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

        let det_f64: f64 = det.into();
        if det_f64.abs() < 1e-12 {
            return Err(PhysicsError::NumericalInstability(
                "Spatial metric determinant is zero".into(),
            ));
        }

        let one = <S as From<f64>>::from(1.0);
        let inv_det = one / det;

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

impl<S> AdmOps<S> for AdmState<S>
where
    S: Field + Clone + Copy + From<f64> + Into<f64>,
{
    fn hamiltonian_constraint(
        &self,
        matter_density: Option<&CausalTensor<S>>,
    ) -> Result<CausalTensor<S>, PhysicsError> {
        // H = R + K² - K_ij K^ij - 16πρ
        let inv_gamma = self.inverse_spatial_metric()?;
        let k_tensor = self.extrinsic_curvature.as_slice();

        let mut k_mixed_trace = S::zero();
        let mut k_sq_contracted = S::zero();

        let mut k_upper = [[S::zero(); 3]; 3];

        for i in 0..3 {
            for j in 0..3 {
                let mut sum = S::zero();
                for a in 0..3 {
                    for b in 0..3 {
                        let k_ab = k_tensor[a * 3 + b];
                        sum = sum + inv_gamma[i][a] * inv_gamma[j][b] * k_ab;
                    }
                }
                k_upper[i][j] = sum;
            }
        }

        for i in 0..3 {
            for j in 0..3 {
                let k_ij = k_tensor[i * 3 + j];
                k_mixed_trace = k_mixed_trace + inv_gamma[i][j] * k_ij;
            }
        }

        for i in 0..3 {
            for j in 0..3 {
                let k_ij = k_tensor[i * 3 + j];
                k_sq_contracted = k_sq_contracted + k_ij * k_upper[i][j];
            }
        }

        let r = self.spatial_ricci_scalar;
        let rho = match matter_density {
            Some(t) => t.as_slice().first().cloned().unwrap_or(S::zero()),
            None => S::zero(),
        };

        let pi_16 = <S as From<f64>>::from(16.0 * std::f64::consts::PI);
        let h = r + (k_mixed_trace * k_mixed_trace) - k_sq_contracted - pi_16 * rho;

        Ok(CausalTensor::from_vec(vec![h], &[1]))
    }

    fn momentum_constraint(
        &self,
        matter_momentum: Option<&CausalTensor<S>>,
    ) -> Result<CausalTensor<S>, PhysicsError> {
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
        let mut k_trace = S::zero();
        for i in 0..3 {
            for j in 0..3 {
                k_trace = k_trace + inv_gamma[i][j] * k_tensor[i * 3 + j];
            }
        }

        // Compute K^j_i = γ^jk K_ki
        let mut k_mixed = [[S::zero(); 3]; 3]; // K^j_i
        for j in 0..3 {
            for i in 0..3 {
                let mut sum = S::zero();
                for k in 0..3 {
                    sum = sum + inv_gamma[j][k] * k_tensor[k * 3 + i];
                }
                k_mixed[j][i] = sum;
            }
        }

        // Compute M_i = D_j (K^j_i - δ^j_i K) - 8πj_i
        let mut m = [S::zero(), S::zero(), S::zero()];

        // Helper closures
        let gamma_trace = |k: usize| -> S {
            let mut sum = S::zero();
            for j in 0..3 {
                sum = sum + gamma_data[j * 9 + j * 3 + k];
            }
            sum
        };

        let gamma = |k: usize, j: usize, i: usize| -> S { gamma_data[k * 9 + j * 3 + i] };

        for (i, m_i) in m.iter_mut().enumerate() {
            // T^j_i = K^j_i - δ^j_i K
            let t = |j: usize, i_inner: usize| -> S {
                let delta = if j == i_inner { S::one() } else { S::zero() };
                k_mixed[j][i_inner] - delta * k_trace
            };

            // Connection terms: Γ^j_jk T^k_i - Γ^k_ji T^j_k
            let mut conn_term = S::zero();
            for k in 0..3 {
                conn_term = conn_term + gamma_trace(k) * t(k, i);
            }
            for k in 0..3 {
                for j in 0..3 {
                    conn_term = conn_term - gamma(k, j, i) * t(j, k);
                }
            }

            // Matter momentum term
            let j_i = match matter_momentum {
                Some(j_tensor) => j_tensor.as_slice().get(i).cloned().unwrap_or(S::zero()),
                None => S::zero(),
            };

            let pi_8 = <S as From<f64>>::from(8.0 * std::f64::consts::PI);
            *m_i = conn_term - pi_8 * j_i;
        }

        Ok(CausalTensor::from_vec(m.to_vec(), &[3]))
    }

    fn mean_curvature(&self) -> Result<CausalTensor<S>, PhysicsError> {
        let inv_gamma = self.inverse_spatial_metric()?;
        let k_tensor = self.extrinsic_curvature.as_slice();

        let zero = <S as From<f64>>::from(0.0);
        let mut k = zero;
        for i in 0..3 {
            for j in 0..3 {
                k = k + inv_gamma[i][j] * k_tensor[i * 3 + j];
            }
        }

        Ok(CausalTensor::from_vec(vec![k], &[1]))
    }
}
