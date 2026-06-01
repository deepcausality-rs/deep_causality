/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Discrete Hodge–Helmholtz decomposition implementation.
//!
//! Given a k-form ω on a `Manifold<K, R>` whose `K::Metric: HasHodgeStar<R>`, the
//! decomposition produces three pairwise-orthogonal components
//!
//! ```text
//! ω = d α  +  δ β  +  h
//! ```
//!
//! via two sequential discrete Poisson solves and a residual computation:
//!
//! 1. Solve `Δ_{k-1} φ_α = δ ω` for the (k-1)-form potential `φ_α`; set `α = d φ_α`.
//! 2. Solve `Δ_{k+1} ψ_β = d ω` for the (k+1)-form potential `ψ_β`; set `β = δ ψ_β`.
//! 3. `h = ω − α − β`.
//!
//! The Poisson solves run through the matrix-free CG solver
//! [`deep_causality_sparse::cg_solve`], which composes against `Manifold::laplacian`
//! rather than assembling `Δ_k` as a `CsrMatrix`. The gauge-fixing mean subtraction
//! lives in [`crate::utils::cg_solver`]. The gauge non-uniqueness at grade 0 (where
//! the constant functions are always harmonic) is fixed by subtracting the mean from
//! `φ_α` per the standard DEC convention.

use core::fmt::{Debug, Display};

use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;

use crate::errors::topology_error::TopologyError;
use crate::traits::chain_complex::ChainComplex;
use crate::traits::has_hodge_star::HasHodgeStar;
use crate::types::hodge_decomposition::HodgeDecomposition;
use crate::types::manifold::Manifold;
use crate::utils::cg_solver::subtract_mean_in_place;
use deep_causality_sparse::{CgFailure, cg_solve};

/// Caller-tunable knobs for `Manifold::hodge_decompose_opts`.
///
/// Both fields are `Option`; `None` means "use the default derived from the precision
/// backend's machine epsilon". The default tolerance is
/// `R::from_f64(1e-10).unwrap_or_else(R::epsilon)`; the default iteration budget is
/// `1000`, adequate for the lattice sizes the downstream fluid pipeline targets
/// (≤ 256³).
#[derive(Debug, Clone)]
pub struct HodgeDecomposeOptions<R: RealField> {
    /// Relative-residual convergence threshold for the iterative solve.
    pub tolerance: Option<R>,
    /// Maximum CG iteration count before reporting non-convergence.
    pub max_iterations: Option<usize>,
}

impl<R: RealField> Default for HodgeDecomposeOptions<R> {
    fn default() -> Self {
        Self {
            tolerance: None,
            max_iterations: None,
        }
    }
}

/// Private control-flow enum carrying typed `R`-precision failure detail inside the
/// `hodge_decompose` impl. Converted to a stringly-typed `TopologyError` at the `Err`
/// boundary so the public error surface stays free of any precision-bearing type
/// parameter. Never crosses the module boundary.
enum HodgeFailReason<R: RealField> {
    Nonconvergence { iterations: usize, residual: R },
    GradeOutOfRange { k: usize, max_dim: usize },
    DimensionMismatch { expected: usize, actual: usize },
    MissingMetric,
}

impl<R: RealField + Display> HodgeFailReason<R> {
    fn into_topology_error(self) -> TopologyError {
        match self {
            Self::Nonconvergence {
                iterations,
                residual,
            } => TopologyError::HodgeDecompositionFailed(format!(
                "iterative solve did not converge in {} iterations (final residual {})",
                iterations, residual
            )),
            Self::GradeOutOfRange { k, max_dim } => TopologyError::HodgeDecompositionFailed(
                format!("grade {} exceeds manifold max_dim {}", k, max_dim),
            ),
            Self::DimensionMismatch { expected, actual } => {
                TopologyError::HodgeDecompositionFailed(format!(
                    "field length {} does not match expected {}",
                    actual, expected
                ))
            }
            Self::MissingMetric => TopologyError::HodgeDecompositionFailed(
                "manifold has no metric attached".to_string(),
            ),
        }
    }
}

impl<K, R> Manifold<K, R>
where
    K: ChainComplex + Clone,
    K::Metric: HasHodgeStar<R, Complex = K> + Clone,
    R: RealField + FromPrimitive + Default + PartialEq + Debug + Display,
{
    /// Discrete Hodge–Helmholtz decomposition of a k-form `field` on this manifold,
    /// using default tolerance and iteration budget.
    ///
    /// See [`HodgeDecomposeOptions`] for the override path and the module-level doc
    /// for the algorithm.
    ///
    /// # Errors
    /// Returns `TopologyErrorEnum::HodgeDecompositionFailed(msg)` if:
    /// - `k > self.complex.max_dim()` (grade out of range),
    /// - `field.len() != self.complex.num_cells(k)` (dimension mismatch),
    /// - the manifold has no metric attached,
    /// - or the iterative solve does not converge within the iteration budget.
    pub fn hodge_decompose(
        &self,
        field: &CausalTensor<R>,
        k: usize,
    ) -> Result<HodgeDecomposition<R>, TopologyError> {
        self.hodge_decompose_opts(field, k, &HodgeDecomposeOptions::default())
    }

    /// Discrete Hodge–Helmholtz decomposition with caller-supplied tolerance and
    /// iteration budget. See [`hodge_decompose`](Self::hodge_decompose) for the
    /// default-options entry point and the failure modes.
    pub fn hodge_decompose_opts(
        &self,
        field: &CausalTensor<R>,
        k: usize,
        opts: &HodgeDecomposeOptions<R>,
    ) -> Result<HodgeDecomposition<R>, TopologyError> {
        let max_dim = self.complex.max_dim();
        if k > max_dim {
            return Err(HodgeFailReason::<R>::GradeOutOfRange { k, max_dim }.into_topology_error());
        }
        let expected_len = self.complex.num_cells(k);
        let actual_len = field.as_slice().len();
        if actual_len != expected_len {
            return Err(HodgeFailReason::<R>::DimensionMismatch {
                expected: expected_len,
                actual: actual_len,
            }
            .into_topology_error());
        }
        if self.metric.is_none() {
            return Err(HodgeFailReason::<R>::MissingMetric.into_topology_error());
        }

        let tolerance = opts.tolerance.unwrap_or_else(|| {
            // Default convergence threshold: tight relative residual (1e-10),
            // but clamped to `R::epsilon() * 100` from below so the threshold
            // stays representable at low-precision backends. At f64 this stays
            // at 1e-10; at f32 (epsilon ≈ 1.19e-7) it floors to ~1.19e-5;
            // at Float106 it floors to ~1e-30. Without this clamp, the f32
            // path tries to converge below its own representable noise.
            let candidate = <R as FromPrimitive>::from_f64(1e-10).unwrap_or_else(R::epsilon);
            let floor = R::epsilon() * <R as FromPrimitive>::from_f64(100.0).unwrap_or(R::one());
            if candidate > floor { candidate } else { floor }
        });
        // Reject non-positive tolerances explicitly. A zero or negative
        // threshold makes the relative-residual convergence check
        // (`rsold.sqrt() < abs_tol`) unreachable: `sqrt()` is non-negative
        // and `tolerance * b_norm` is non-positive, so CG would run to the
        // iteration cap and surface a misleading `Nonconvergence` error
        // whose actual cause is a caller-supplied invalid threshold.
        if tolerance <= R::zero() {
            return Err(TopologyError::HodgeDecompositionFailed(format!(
                "tolerance must be strictly positive, got {}",
                tolerance
            )));
        }
        let max_iter = opts.max_iterations.unwrap_or(1000);

        let omega: Vec<R> = field.as_slice().to_vec();
        let n_k = expected_len;

        // --- α = d φ_α with Δ_{k-1} φ_α = δω ---
        let alpha: Vec<R> = if k == 0 {
            vec![R::zero(); n_k]
        } else {
            let n_km1 = self.complex.num_cells(k - 1);
            let omega_tensor = CausalTensor::new(omega.clone(), vec![n_k]).unwrap();
            let temp_omega = self.create_temp_manifold(k, omega_tensor);
            let delta_omega = temp_omega.codifferential(k);
            let mut rhs = delta_omega.as_slice().to_vec();
            pad_or_truncate(&mut rhs, n_km1);
            if k - 1 == 0 {
                subtract_mean_in_place(&mut rhs);
            }

            let mut phi_alpha = solve_laplacian(self, k - 1, &rhs, tolerance, max_iter)?;
            if k - 1 == 0 {
                subtract_mean_in_place(&mut phi_alpha);
            }

            let phi_tensor = CausalTensor::new(phi_alpha, vec![n_km1]).unwrap();
            let temp_phi = self.create_temp_manifold(k - 1, phi_tensor);
            let alpha_tensor = temp_phi.exterior_derivative(k - 1);
            let mut alpha = alpha_tensor.as_slice().to_vec();
            pad_or_truncate(&mut alpha, n_k);
            alpha
        };

        // --- β = δ ψ_β with Δ_{k+1} ψ_β = dω ---
        let beta: Vec<R> = if k >= max_dim {
            vec![R::zero(); n_k]
        } else {
            let n_kp1 = self.complex.num_cells(k + 1);
            let omega_tensor = CausalTensor::new(omega.clone(), vec![n_k]).unwrap();
            let temp_omega = self.create_temp_manifold(k, omega_tensor);
            let d_omega = temp_omega.exterior_derivative(k);
            let mut rhs = d_omega.as_slice().to_vec();
            pad_or_truncate(&mut rhs, n_kp1);

            let psi_beta = solve_laplacian(self, k + 1, &rhs, tolerance, max_iter)?;

            let psi_tensor = CausalTensor::new(psi_beta, vec![n_kp1]).unwrap();
            let temp_psi = self.create_temp_manifold(k + 1, psi_tensor);
            let beta_tensor = temp_psi.codifferential(k + 1);
            let mut beta = beta_tensor.as_slice().to_vec();
            pad_or_truncate(&mut beta, n_k);
            beta
        };

        // --- h = ω − α − β ---
        let harmonic: Vec<R> = (0..n_k).map(|i| omega[i] - alpha[i] - beta[i]).collect();

        let exact_t = CausalTensor::new(alpha, vec![n_k]).unwrap();
        let co_exact_t = CausalTensor::new(beta, vec![n_k]).unwrap();
        let harmonic_t = CausalTensor::new(harmonic, vec![n_k]).unwrap();

        Ok(HodgeDecomposition::new(exact_t, co_exact_t, harmonic_t, k))
    }
}

/// Solve `Δ_grade x = rhs` matrix-free via CG on this manifold.
fn solve_laplacian<K, R>(
    manifold: &Manifold<K, R>,
    grade: usize,
    rhs: &[R],
    tolerance: R,
    max_iter: usize,
) -> Result<Vec<R>, TopologyError>
where
    K: ChainComplex + Clone,
    K::Metric: HasHodgeStar<R, Complex = K> + Clone,
    R: RealField + FromPrimitive + Default + PartialEq + Debug + Display,
{
    let n = rhs.len();
    let apply = |v: &[R]| -> Vec<R> {
        let tensor = CausalTensor::new(v.to_vec(), vec![v.len()]).unwrap();
        let temp = manifold.create_temp_manifold(grade, tensor);
        let result = temp.laplacian(grade);
        let mut out = result.as_slice().to_vec();
        pad_or_truncate(&mut out, n);
        out
    };
    match cg_solve(apply, rhs, tolerance, max_iter) {
        Ok(x) => Ok(x),
        Err(CgFailure {
            iterations,
            residual,
        }) => Err(HodgeFailReason::Nonconvergence {
            iterations,
            residual,
        }
        .into_topology_error()),
    }
}

fn pad_or_truncate<R: RealField>(v: &mut Vec<R>, target_len: usize) {
    if v.len() < target_len {
        v.resize(target_len, R::zero());
    } else if v.len() > target_len {
        v.truncate(target_len);
    }
}
