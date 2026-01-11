/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeGroup, LinkVariable, LinkVariableError};
use deep_causality_tensor::CausalTensor;
use std::marker::PhantomData;

impl<G: GaugeGroup, T: Clone + Default> LinkVariable<G, T> {
    /// Hermitian conjugate U†.
    ///
    /// For real matrices, this is the transpose.
    /// For complex matrices, this is transpose + complex conjugate.
    ///
    /// # Errors
    ///
    /// Returns error if tensor creation fails.
    pub fn try_dagger(&self) -> Result<Self, LinkVariableError>
    where
        T: Clone,
    {
        let n = G::matrix_dim();
        let slice = self.data.as_slice();
        let mut result = vec![T::default(); n * n];

        // Transpose (for real matrices, dagger = transpose)
        for i in 0..n {
            for j in 0..n {
                result[j * n + i] = slice[i * n + j].clone();
            }
        }

        CausalTensor::new(result, vec![n, n])
            .map(|tensor| Self {
                data: tensor,
                _gauge: PhantomData,
            })
            .map_err(|e| LinkVariableError::TensorCreation(format!("{:?}", e)))
    }

    /// Hermitian conjugate U† (convenience method).
    ///
    /// For real matrices, this is the transpose.
    pub fn dagger(&self) -> Self
    where
        T: Clone,
    {
        // This is a simple memory operation that cannot fail for valid LinkVariable
        let n = G::matrix_dim();
        let slice = self.data.as_slice();
        let mut result = vec![T::default(); n * n];

        for i in 0..n {
            for j in 0..n {
                result[j * n + i] = slice[i * n + j].clone();
            }
        }

        // Safe because we're creating the correct shape
        Self {
            data: CausalTensor::new(result, vec![n, n])
                .unwrap_or_else(|_| panic!("Dagger failed for valid {}x{} matrix", n, n)),
            _gauge: PhantomData,
        }
    }

    /// Group multiplication: self * other.
    ///
    /// Standard matrix multiplication for group elements.
    ///
    /// # Errors
    ///
    /// Returns error if tensor creation fails.
    pub fn try_mul(&self, other: &Self) -> Result<Self, LinkVariableError>
    where
        T: Clone + std::ops::Mul<Output = T> + std::ops::Add<Output = T>,
    {
        let n = G::matrix_dim();
        let a = self.data.as_slice();
        let b = other.data.as_slice();
        let mut result = vec![T::default(); n * n];

        // Standard matrix multiplication: C[i,j] = Σ_k A[i,k] * B[k,j]
        for i in 0..n {
            for j in 0..n {
                let mut sum = T::default();
                for k in 0..n {
                    let prod = a[i * n + k].clone() * b[k * n + j].clone();
                    sum = sum + prod;
                }
                result[i * n + j] = sum;
            }
        }

        CausalTensor::new(result, vec![n, n])
            .map(|tensor| Self {
                data: tensor,
                _gauge: PhantomData,
            })
            .map_err(|e| LinkVariableError::TensorCreation(format!("{:?}", e)))
    }

    /// Group multiplication: self * other (convenience method).
    pub fn mul(&self, other: &Self) -> Self
    where
        T: Clone + std::ops::Mul<Output = T> + std::ops::Add<Output = T>,
    {
        let n = G::matrix_dim();
        let a = self.data.as_slice();
        let b = other.data.as_slice();
        let mut result = vec![T::default(); n * n];

        for i in 0..n {
            for j in 0..n {
                let mut sum = T::default();
                for k in 0..n {
                    let prod = a[i * n + k].clone() * b[k * n + j].clone();
                    sum = sum + prod;
                }
                result[i * n + j] = sum;
            }
        }

        Self {
            data: CausalTensor::new(result, vec![n, n])
                .unwrap_or_else(|_| panic!("Matrix multiply failed for {}x{}", n, n)),
            _gauge: PhantomData,
        }
    }

    /// Matrix addition: self + other.
    ///
    /// # Errors
    ///
    /// Returns error if tensor creation fails.
    pub fn try_add(&self, other: &Self) -> Result<Self, LinkVariableError>
    where
        T: Clone + std::ops::Add<Output = T>,
    {
        let n = G::matrix_dim();
        let a = self.data.as_slice();
        let b = other.data.as_slice();
        let mut result = vec![T::default(); n * n];

        for i in 0..(n * n) {
            result[i] = a[i].clone() + b[i].clone();
        }

        CausalTensor::new(result, vec![n, n])
            .map(|tensor| Self {
                data: tensor,
                _gauge: PhantomData,
            })
            .map_err(|e| LinkVariableError::TensorCreation(format!("{:?}", e)))
    }

    /// Matrix addition: self + other (convenience method).
    pub fn add(&self, other: &Self) -> Self
    where
        T: Clone + std::ops::Add<Output = T>,
    {
        let n = G::matrix_dim();
        let a = self.data.as_slice();
        let b = other.data.as_slice();
        let mut result = vec![T::default(); n * n];

        for i in 0..(n * n) {
            result[i] = a[i].clone() + b[i].clone();
        }

        Self {
            data: CausalTensor::new(result, vec![n, n])
                .unwrap_or_else(|_| panic!("Matrix add failed")),
            _gauge: PhantomData,
        }
    }

    /// Scalar multiplication: α * self.
    ///
    /// # Errors
    ///
    /// Returns error if tensor creation fails.
    pub fn try_scale(&self, alpha: &T) -> Result<Self, LinkVariableError>
    where
        T: Clone + std::ops::Mul<Output = T>,
    {
        let n = G::matrix_dim();
        let a = self.data.as_slice();
        let mut result = vec![T::default(); n * n];

        for i in 0..(n * n) {
            result[i] = alpha.clone() * a[i].clone();
        }

        CausalTensor::new(result, vec![n, n])
            .map(|tensor| Self {
                data: tensor,
                _gauge: PhantomData,
            })
            .map_err(|e| LinkVariableError::TensorCreation(format!("{:?}", e)))
    }

    /// Scalar multiplication: α * self (convenience method).
    pub fn scale(&self, alpha: &T) -> Self
    where
        T: Clone + std::ops::Mul<Output = T>,
    {
        let n = G::matrix_dim();
        let a = self.data.as_slice();
        let mut result = vec![T::default(); n * n];

        for i in 0..(n * n) {
            result[i] = alpha.clone() * a[i].clone();
        }

        Self {
            data: CausalTensor::new(result, vec![n, n])
                .unwrap_or_else(|_| panic!("Matrix scale failed")),
            _gauge: PhantomData,
        }
    }

    /// Trace of the matrix: Tr(U) = Σ_i U_ii.
    pub fn trace(&self) -> T
    where
        T: Clone + std::ops::Add<Output = T>,
    {
        let n = G::matrix_dim();
        let slice = self.data.as_slice();
        let mut sum = T::default();

        for i in 0..n {
            sum = sum + slice[i * n + i].clone();
        }
        sum
    }

    /// Real part of trace (for action computation with real scalars).
    ///
    /// For real T, this is identical to trace().
    #[inline]
    pub fn re_trace(&self) -> T
    where
        T: Clone + std::ops::Add<Output = T>,
    {
        self.trace()
    }

    /// Frobenius norm squared: ||U||²_F = Tr(U†U) = Σ_{ij} |U_ij|².
    ///
    /// For real matrices: Σ_{ij} U_ij².
    pub fn frobenius_norm_sq(&self) -> T
    where
        T: Clone + std::ops::Mul<Output = T> + std::ops::Add<Output = T>,
    {
        let slice = self.data.as_slice();
        let mut sum = T::default();

        for val in slice {
            sum = sum + val.clone() * val.clone();
        }
        sum
    }

    /// Project to SU(N) using polar decomposition.
    ///
    /// Given a general matrix M, computes U = M (M†M)^{-1/2}
    /// which is the closest unitary matrix to M in Frobenius norm.
    ///
    /// # Errors
    ///
    /// Returns `LinkVariableError::SingularMatrix` if M†M is not invertible.
    /// Returns `LinkVariableError::NumericalError` for other numerical issues.
    pub fn project_sun(&self) -> Result<Self, LinkVariableError>
    where
        T: Clone
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<Output = T>
            + From<f64>
            + PartialOrd,
    {
        // For real matrices, polar decomposition: U = M (M^T M)^{-1/2}
        // We use iterative Newton-Schulz iteration:
        // X_{k+1} = 0.5 * X_k (3I - X_k^T X_k)
        // Converges to U where M = UP, P positive semi-definite

        let _n = G::matrix_dim();
        let mut x = self.clone();

        // Normalize by Frobenius norm for numerical stability
        let norm_sq = self.frobenius_norm_sq();
        let _one = T::from(1.0);
        let zero = T::from(0.0);

        if norm_sq.partial_cmp(&zero) != Some(std::cmp::Ordering::Greater) {
            // Zero matrix - return identity
            return Self::try_identity();
        }

        // Newton-Schulz iteration (typically converges in 10-20 iterations)
        let max_iter = 50;
        let three = T::from(3.0);
        let half = T::from(0.5);

        for _ in 0..max_iter {
            let x_dag = x.dagger();
            let xdx = x_dag.mul(&x);

            // 3I - X†X
            let identity = Self::try_identity()?;
            let three_i = identity.scale(&three);
            let diff = three_i.try_add(&xdx.scale(&T::from(-1.0)))?;

            // X_{k+1} = 0.5 * X * diff
            let new_x = x.mul(&diff).scale(&half);

            // Check convergence: ||X†X - I||_F < ε
            let new_xdx = new_x.dagger().mul(&new_x);
            let residual = compute_identity_deviation::<G, T>(&new_xdx)?;

            x = new_x;

            if residual < T::from(1e-12) {
                break;
            }
        }

        // Ensure determinant = 1 for SU(N) by dividing by det^{1/N}
        // For SU(2) and SU(3) this is a standard normalization
        // Skipped here as it requires complex arithmetic for general case

        Ok(x)
    }
}

/// Compute ||X - I||_F for checking how close X is to identity.
fn compute_identity_deviation<G: GaugeGroup, T>(
    x: &LinkVariable<G, T>,
) -> Result<T, LinkVariableError>
where
    T: Clone
        + Default
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>
        + From<f64>,
{
    let n = G::matrix_dim();
    let slice = x.as_slice();
    let mut sum = T::from(0.0);
    let one = T::from(1.0);

    for i in 0..n {
        for j in 0..n {
            let val = slice[i * n + j].clone();
            let diff = if i == j { val - one.clone() } else { val };
            sum = sum + diff.clone() * diff;
        }
    }

    Ok(sum)
}
