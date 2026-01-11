/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Link variable type for lattice gauge theory.
//!
//! A link variable U_μ(x) ∈ G is an element of the gauge group assigned
//! to the edge connecting lattice site x to x + μ̂.
//!

use crate::{GaugeGroup, LinkVariableError};
use deep_causality_tensor::{CausalTensor, TensorData};
use std::marker::PhantomData;

mod display;
mod getters;
mod ops;
mod part_eq;

/// A link variable U_μ(n) ∈ G on a lattice edge.
///
/// For SU(N), this is an N×N unitary matrix with det = 1.
/// The matrix is stored as a flattened tensor.
///
/// # Type Parameters
///
/// * `G` - The gauge group (U1, SU2, SU3, etc.)
/// * `T` - Scalar type for matrix elements (typically f64 or Complex<f64>)
///
/// # Mathematical Properties
///
/// - **Unitarity:** U U† = 1
/// - **Orientation:** U_{-μ}(n + μ̂) = U_μ(n)†
/// - **Continuum limit:** U_μ(x) ≈ exp(ia A_μ(x))
#[derive(Debug, Clone)]
pub struct LinkVariable<G: GaugeGroup, T> {
    /// Matrix elements of the group element.
    /// Shape: [N, N] for SU(N) where N = matrix dimension.
    data: CausalTensor<T>,
    _gauge: PhantomData<G>,
}

impl<G: GaugeGroup, T: TensorData> LinkVariable<G, T> {
    /// Create the identity link (unit element of G).
    ///
    /// Returns the N×N identity matrix for SU(N).
    ///
    /// # Returns
    ///
    /// Identity link variable.
    ///
    /// # Errors
    ///
    /// Returns `LinkVariableError::TensorCreation` if matrix allocation fails.
    pub fn try_identity() -> Result<Self, LinkVariableError>
    where
        T: From<f64>,
    {
        let n = G::matrix_dim();
        if n == 0 {
            return Err(LinkVariableError::InvalidDimension(n));
        }

        let mut data = vec![T::from(0.0); n * n];
        // Set diagonal to 1
        for i in 0..n {
            data[i * n + i] = T::from(1.0);
        }

        CausalTensor::new(data, vec![n, n])
            .map(|tensor| Self {
                data: tensor,
                _gauge: PhantomData,
            })
            .map_err(|e| LinkVariableError::TensorCreation(format!("{:?}", e)))
    }

    /// Create the identity link (unit element of G).
    ///
    /// Convenience method that panics on failure. Use `try_identity()` for
    /// fallible construction.
    ///
    /// # Panics
    ///
    /// Panics if tensor creation fails (should never happen for valid groups).
    pub fn identity() -> Self
    where
        T: From<f64>,
    {
        Self::try_identity()
            .unwrap_or_else(|e| panic!("Identity matrix creation failed for {}: {}", G::name(), e))
    }

    /// Create from raw matrix data with validation.
    ///
    /// # Arguments
    ///
    /// * `data` - Tensor of shape [N, N] for SU(N)
    ///
    /// # Returns
    ///
    /// The wrapped link variable.
    ///
    /// # Errors
    ///
    /// Returns `LinkVariableError::ShapeMismatch` if tensor shape doesn't match
    /// expected [N, N] for the gauge group.
    pub fn try_from_matrix(data: CausalTensor<T>) -> Result<Self, LinkVariableError> {
        let n = G::matrix_dim();
        let expected = vec![n, n];
        let got = data.shape().to_vec();

        if got != expected {
            return Err(LinkVariableError::ShapeMismatch { expected, got });
        }

        Ok(Self {
            data,
            _gauge: PhantomData,
        })
    }

    /// Create from raw matrix data without validation.
    ///
    /// # Safety
    ///
    /// Caller must ensure the tensor has correct shape [N, N].
    pub fn from_matrix_unchecked(data: CausalTensor<T>) -> Self {
        Self {
            data,
            _gauge: PhantomData,
        }
    }

    /// Create a zero matrix.
    ///
    /// # Errors
    ///
    /// Returns error if tensor creation fails.
    pub fn try_zero() -> Result<Self, LinkVariableError>
    where
        T: From<f64>,
    {
        let n = G::matrix_dim();
        if n == 0 {
            return Err(LinkVariableError::InvalidDimension(n));
        }

        let data = vec![T::from(0.0); n * n];
        CausalTensor::new(data, vec![n, n])
            .map(|tensor| Self {
                data: tensor,
                _gauge: PhantomData,
            })
            .map_err(|e| LinkVariableError::TensorCreation(format!("{:?}", e)))
    }

    /// Create a random link variable for Monte Carlo initialization.
    ///
    /// Generates a random group element suitable for "hot start" Monte Carlo
    /// simulations, where configurations begin far from equilibrium.
    ///
    /// # Mathematics
    ///
    /// For SU(N), generates a random unitary matrix by:
    /// 1. Creating a random N×N matrix M with entries uniform in [-1, 1]
    /// 2. Projecting to SU(N) via `project_sun()` (polar decomposition)
    ///
    /// The resulting matrix satisfies:
    /// - **Unitarity:** U U† = 𝟙
    /// - **Special:** det(U) = 1 (for SU(N))
    ///
    /// # Physics
    ///
    /// Random link variables represent a "hot start" configuration for
    /// lattice gauge theory simulations. The configuration is far from
    /// the classical vacuum and requires thermalization before measurements.
    ///
    /// # Arguments
    ///
    /// * `rng` - Random number generator implementing `deep_causality_rand::Rng`
    ///
    /// # Returns
    ///
    /// A random SU(N) link variable.
    ///
    /// # Errors
    ///
    /// Returns `LinkVariableError::TensorCreation` if matrix allocation fails,
    /// or `LinkVariableError::SingularMatrix` if projection fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use deep_causality_topology::{LinkVariable, SU2};
    /// use deep_causality_rand::rng;
    ///
    /// let mut rng = rng();
    /// let link: LinkVariable<SU2, f64> = LinkVariable::try_random(&mut rng)?;
    /// ```
    pub fn try_random<R>(rng: &mut R) -> Result<Self, LinkVariableError>
    where
        R: deep_causality_rand::Rng,
        T: From<f64> + PartialOrd,
    {
        let n = G::matrix_dim();
        if n == 0 {
            return Err(LinkVariableError::InvalidDimension(n));
        }

        // Generate random matrix with entries in [-1, 1]
        let mut data = Vec::with_capacity(n * n);
        for _ in 0..(n * n) {
            // Generate uniform in [0, 1), scale to [-0.5, 0.5)
            // Smaller range ensures initial matrix is within Newton-Schulz convergence radius
            let val: f64 = rng.random();
            data.push(T::from(val - 0.5));
        }

        let tensor = CausalTensor::new(data, vec![n, n])
            .map_err(|e| LinkVariableError::TensorCreation(format!("{:?}", e)))?;

        let random_matrix = Self {
            data: tensor,
            _gauge: PhantomData,
        };

        // Project to SU(N) to ensure unitarity and det = 1
        random_matrix.project_sun()
    }

    /// Create a random link variable (convenience method).
    ///
    /// See [`try_random`](Self::try_random) for details.
    ///
    /// # Panics
    ///
    /// Panics if random matrix creation or SU(N) projection fails.
    pub fn random<R>(rng: &mut R) -> Self
    where
        R: deep_causality_rand::Rng,
        T: From<f64> + PartialOrd,
    {
        Self::try_random(rng)
            .unwrap_or_else(|e| panic!("Random link creation failed for {}: {}", G::name(), e))
    }
}
