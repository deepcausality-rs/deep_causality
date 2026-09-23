/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeGroup, LinkVariable, LinkVariableError};
use deep_causality_algebra::{ComplexField, Field, RealField};
use deep_causality_num::FromPrimitive;
use std::fmt::Debug;
use std::marker::PhantomData;

impl<
    G: GaugeGroup,
    M: Field + Copy + Default + PartialOrd + Debug,
    R: RealField + FromPrimitive + deep_causality_num::ToPrimitive,
> LinkVariable<G, M, R>
{
    /// Hermitian conjugate U†.
    ///
    /// For real matrices, this is the transpose.
    /// For complex matrices, this is transpose + complex conjugate.
    pub fn dagger(&self) -> Self
    where
        M: ComplexField<R>,
        R: RealField,
    {
        let n = G::matrix_dim();
        let slice = self.data.as_slice();
        let mut result = vec![M::default(); n * n];

        for i in 0..n {
            for j in 0..n {
                result[j * n + i] = slice[i * n + j].conjugate();
            }
        }

        Self {
            data: result,
            _gauge: PhantomData,
            _scalar: PhantomData,
        }
    }

    /// Group multiplication: self * other.
    ///
    /// Standard matrix multiplication for group elements.
    ///
    /// # Arguments
    ///
    /// * `other` - The matrix to multiply with (on the right)
    ///
    /// # Returns
    ///
    /// The product $U \cdot V$.
    pub fn mul(&self, other: &Self) -> Self
    where
        M: Field,
    {
        let n = G::matrix_dim();
        let a = self.data.as_slice();
        let b = other.data.as_slice();
        let mut result = vec![M::default(); n * n];

        for i in 0..n {
            for j in 0..n {
                let mut sum = M::default();
                for k in 0..n {
                    let prod = a[i * n + k] * b[k * n + j];
                    sum = sum + prod;
                }
                result[i * n + j] = sum;
            }
        }

        Self {
            data: result,
            _gauge: PhantomData,
            _scalar: PhantomData,
        }
    }

    /// Matrix addition: self + other.
    pub fn add(&self, other: &Self) -> Self
    where
        M: Field,
    {
        let n = G::matrix_dim();
        let a = self.data.as_slice();
        let b = other.data.as_slice();
        let mut result = vec![M::default(); n * n];

        for i in 0..(n * n) {
            result[i] = a[i] + b[i];
        }

        Self {
            data: result,
            _gauge: PhantomData,
            _scalar: PhantomData,
        }
    }

    /// Scalar multiplication: α * self.
    pub fn scale(&self, alpha: &M) -> Self
    where
        M: Field,
    {
        let n = G::matrix_dim();
        let a = self.data.as_slice();
        let mut result = vec![M::default(); n * n];

        for i in 0..(n * n) {
            result[i] = *alpha * a[i];
        }

        Self {
            data: result,
            _gauge: PhantomData,
            _scalar: PhantomData,
        }
    }

    /// Trace of the matrix: Tr(U) = Σ_i U_ii.
    pub fn trace(&self) -> M
    where
        M: Field,
    {
        let n = G::matrix_dim();
        let slice = self.data.as_slice();
        let mut sum = M::default();

        for i in 0..n {
            sum = sum + slice[i * n + i];
        }
        sum
    }

    /// Real part of trace (for action computation with real scalars).
    ///
    /// Returns the real component R of the trace.
    #[inline]
    pub fn re_trace(&self) -> R
    where
        M: ComplexField<R>,
        R: RealField,
    {
        self.trace().real()
    }

    /// Frobenius norm squared: ||U||²_F = Tr(U†U) = Σ_{ij} |U_ij|².
    ///
    /// For real matrices: Σ_{ij} U_ij².
    /// For complex matrices: Σ_{ij} |z_ij|² (returns R).
    pub fn frobenius_norm_sq(&self) -> R
    where
        M: ComplexField<R>,
        R: RealField,
    {
        let slice = self.data.as_slice();
        let mut sum = R::zero();

        for val in slice {
            // Use norm_sqr() to get real-valued squared norm |z|^2
            sum += val.norm_sqr();
        }
        sum
    }

    /// Project to SU(N) using polar decomposition.
    ///
    /// Given a general matrix M, computes U = M (M†M)^{-1/2}
    /// which is the closest unitary matrix to M in Frobenius norm. The zero matrix projects to
    /// the identity.
    ///
    /// # Returns
    ///
    /// The projected SU(N) matrix.
    ///
    /// # Errors
    ///
    /// Returns `LinkVariableError::InvalidDimension` if `G::matrix_dim()` is zero.
    /// Returns `LinkVariableError::NumericalError` if a numeric constant does not convert to `R`.
    pub fn project_sun(&self) -> Result<Self, LinkVariableError>
    where
        M: ComplexField<R>,
        R: RealField,
    {
        // For real matrices, polar decomposition: U = M (M^T M)^{-1/2}
        // We use iterative Newton-Schulz iteration:
        // X_{k+1} = 0.5 * X_k (3I - X_k^T X_k)
        // Converges to U where M = UP, P positive semi-definite

        let mut x = self.clone();
        let epsilon = R::from_f64(1e-24).ok_or_else(|| {
            LinkVariableError::NumericalError("Failed to convert 1e-24 to T".to_string())
        })?;

        // Normalize by Frobenius norm for numerical stability
        // norm_sq is R
        let norm_sq = self.frobenius_norm_sq();
        let zero = R::zero();

        if norm_sq.partial_cmp(&zero) != Some(std::cmp::Ordering::Greater) {
            // Zero matrix - return identity
            return Self::try_identity();
        }

        // inv_norm is R
        let inv_norm = R::one() / norm_sq.sqrt();
        // Convert R to M for scaling
        let inv_norm_m = M::from_re_im(inv_norm, R::zero());
        x = x.scale(&inv_norm_m);

        // Newton-Schulz iteration (typically converges in 10-20 iterations)
        let max_iter = 50;
        let three_r = R::from_f64(3.0).ok_or_else(|| {
            LinkVariableError::NumericalError("Failed to convert 3.0 to T".to_string())
        })?;
        let half_r = R::from_f64(0.5).ok_or_else(|| {
            LinkVariableError::NumericalError("Failed to convert 0.5 to T".to_string())
        })?;

        // Convert to M
        let three_m = M::from_re_im(three_r, R::zero());
        let half_m = M::from_re_im(half_r, R::zero());
        let minus_one_m = M::from_re_im(-R::one(), R::zero());

        for _ in 0..max_iter {
            let x_dag = x.dagger();
            let xdx = x_dag.mul(&x);

            // Check convergence before next iteration (compute_identity_deviation returns ||X-I||_F^2)
            let residual_sq = compute_identity_deviation::<G, M, R>(&xdx);
            if residual_sq < epsilon {
                break;
            }

            // 3I - X†X
            let identity = Self::try_identity()?;
            let three_i = identity.scale(&three_m);
            // xdx * -1
            let xdx_neg = xdx.scale(&minus_one_m);
            let diff = three_i.add(&xdx_neg);

            // X_{k+1} = 0.5 * X * diff
            // order: X * diff * 0.5
            x = x.mul(&diff).scale(&half_m);
        }

        // Ensure determinant = 1 for SU(N) by dividing by det^{1/N}
        // This is only required for non-Abelian groups like SU(2), SU(3)
        // Abelian groups like U(1) are unitary U(1) = circle group, det=u, so u/det^1 = 1 which is wrong
        // So we only apply this if N >= 2
        let n = G::matrix_dim();
        if n >= 2 {
            let det = x.determinant();
            // Compute phase factor to remove: alpha = det^{-1/N}
            // det = r * exp(i * theta) -> because it's unitary, r=1
            // det^{-1/N} = exp(-i * theta / N)

            // arg() returns R
            let theta = det.arg();
            let n_r = R::from_usize(n).ok_or_else(|| {
                LinkVariableError::NumericalError("Failed to convert N to T".to_string())
            })?;
            let theta_norm = theta / n_r;

            // Correction factor: exp(-i * theta/N)
            // cis(-theta) = cos(-theta) + i sin(-theta)
            let neg_theta = -theta_norm;
            let phase_correction = M::from_polar(R::one(), neg_theta);

            // Apply correction
            x = x.scale(&phase_correction);
        }

        Ok(x)
    }

    /// Determinant of the `N x N` matrix.
    ///
    /// Closed forms for `N <= 3`. Above that, LU elimination with partial pivoting: at each
    /// column the row with the largest `|z|²` becomes the pivot, the determinant is the product
    /// of the pivots, and each row swap negates it. A column with no non-zero pivot candidate
    /// makes the determinant exactly zero. `N = 0` gives one, the empty product.
    pub fn determinant(&self) -> M
    where
        M: ComplexField<R>,
    {
        let n = G::matrix_dim();
        let s = self.as_slice();

        match n {
            0 => M::one(),
            1 => s[0],
            2 => {
                // | a b |
                // | c d |
                // det = ad - bc
                let a = s[0];
                let b = s[1];
                let c = s[2];
                let d = s[3];
                a * d - b * c
            }
            3 => {
                // Rule of Sarrus
                let m00 = s[0];
                let m01 = s[1];
                let m02 = s[2];
                let m10 = s[3];
                let m11 = s[4];
                let m12 = s[5];
                let m20 = s[6];
                let m21 = s[7];
                let m22 = s[8];

                let term1 = m00 * m11 * m22;
                let term2 = m01 * m12 * m20;
                let term3 = m02 * m10 * m21;

                let term4 = m02 * m11 * m20;
                let term5 = m01 * m10 * m22;
                let term6 = m00 * m12 * m21;

                term1 + term2 + term3 - term4 - term5 - term6
            }
            _ => lu_determinant::<M, R>(s.to_vec(), n),
        }
    }
}

/// Determinant of the row-major `n x n` matrix `work` by LU elimination with partial pivoting.
fn lu_determinant<M, R>(mut work: Vec<M>, n: usize) -> M
where
    M: ComplexField<R> + Copy,
    R: RealField,
{
    let mut det = M::one();
    for col in 0..n {
        // The row at or below `col` whose entry in this column has the largest modulus.
        let (pivot_row, pivot_norm) = (col..n).map(|r| (r, work[r * n + col].norm_sqr())).fold(
            (col, R::zero()),
            |best, cand| {
                if cand.1 > best.1 { cand } else { best }
            },
        );
        if pivot_norm <= R::zero() {
            return M::zero();
        }
        if pivot_row != col {
            for c in 0..n {
                work.swap(col * n + c, pivot_row * n + c);
            }
            det = M::zero() - det;
        }
        let head = work[col * n + col];
        det = det * head;
        for r in (col + 1)..n {
            let factor = work[r * n + col] / head;
            for c in col..n {
                work[r * n + c] = work[r * n + c] - factor * work[col * n + c];
            }
        }
    }
    det
}

/// Compute ||X - I||_F for checking how close X is to identity.
fn compute_identity_deviation<G: GaugeGroup, M, R>(x: &LinkVariable<G, M, R>) -> R
where
    M: ComplexField<R> + Debug + Copy,
    R: RealField,
{
    let n = G::matrix_dim();
    let slice = x.as_slice();
    let mut sum = R::zero();
    let one = M::one();

    for i in 0..n {
        for j in 0..n {
            let val = slice[i * n + j];
            let diff = if i == j { val - one } else { val };
            // norm_sqr returns R
            self_diff_sq(diff, &mut sum);
        }
    }

    sum
}

#[inline]
fn self_diff_sq<M: ComplexField<R>, R: RealField>(diff: M, sum: &mut R) {
    *sum += diff.norm_sqr();
}

// Helpers for generic float math
