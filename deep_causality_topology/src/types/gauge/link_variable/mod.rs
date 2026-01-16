/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Link variable type for lattice gauge theory.
//!
//! A link variable U_μ(x) ∈ G is an element of the gauge group assigned
//! to the edge connecting lattice site x to x + μ̂.
//!

use crate::types::gauge::link_variable::random::RandomField;
use crate::{GaugeGroup, LinkVariableError};
use deep_causality_num::{
    ComplexField, DivisionAlgebra, Field, FromPrimitive, RealField, ToPrimitive,
};
use deep_causality_tensor::CausalTensor;
use std::marker::PhantomData;

mod display;
mod getters;
mod ops;
mod part_eq;
pub(crate) mod random;

/// A link variable U_μ(n) ∈ G on a lattice edge.
///
/// For SU(N), this is an N×N unitary matrix with det = 1.
/// The matrix is stored as a flattened tensor.
///
/// # Type Parameters
///
/// * `G` - The gauge group (U1, SU2, SU3, etc.)
/// * `G` - The gauge group (U1, SU2, SU3, etc.)
/// * `M` - Matrix element type (Field + `DivisionAlgebra<R>`)
/// * `R` - Scalar type (RealField)
///
/// # Mathematical Properties
///
/// - **Unitarity:** U U† = 1
/// - **Orientation:** U_{-μ}(n + μ̂) = U_μ(n)†
/// - **Continuum limit:** U_μ(x) ≈ exp(ia A_μ(x))
#[derive(Debug, Clone)]
pub struct LinkVariable<G: GaugeGroup, M, R> {
    /// Matrix elements of the group element.
    /// Shape: [N, N] for SU(N) where N = matrix dimension.
    data: CausalTensor<M>,
    _gauge: PhantomData<G>,
    _scalar: PhantomData<R>,
}

impl<G: GaugeGroup, M: Field + Copy + Default + PartialOrd, R: RealField> LinkVariable<G, M, R> {
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
        M: Field,
    {
        let n = G::matrix_dim();
        if n == 0 {
            return Err(LinkVariableError::InvalidDimension(n));
        }

        let mut data = vec![M::zero(); n * n];
        // Set diagonal to 1
        for i in 0..n {
            data[i * n + i] = M::one();
        }

        CausalTensor::new(data, vec![n, n])
            .map(|tensor| Self {
                data: tensor,
                _gauge: PhantomData,
                _scalar: PhantomData,
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
        M: Field,
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
    pub fn try_from_matrix(data: CausalTensor<M>) -> Result<Self, LinkVariableError> {
        let n = G::matrix_dim();
        let expected = vec![n, n];
        let got = data.shape().to_vec();

        if got != expected {
            return Err(LinkVariableError::ShapeMismatch { expected, got });
        }

        Ok(Self {
            data,
            _gauge: PhantomData,
            _scalar: PhantomData,
        })
    }

    /// Create from raw matrix data without validation.
    ///
    /// # Safety
    ///
    /// Caller must ensure the tensor has correct shape [N, N].
    pub fn from_matrix_unchecked(data: CausalTensor<M>) -> Self {
        Self {
            data,
            _gauge: PhantomData,
            _scalar: PhantomData,
        }
    }

    /// Create a zero matrix.
    ///
    /// # Errors
    ///
    /// Returns error if tensor creation fails.
    pub fn try_zero() -> Result<Self, LinkVariableError>
    where
        M: Field,
    {
        let n = G::matrix_dim();
        if n == 0 {
            return Err(LinkVariableError::InvalidDimension(n));
        }

        let data = vec![M::zero(); n * n];
        CausalTensor::new(data, vec![n, n])
            .map(|tensor| Self {
                data: tensor,
                _gauge: PhantomData,
                _scalar: PhantomData,
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
    pub fn try_random<RngType>(rng: &mut RngType) -> Result<Self, LinkVariableError>
    where
        RngType: deep_causality_rand::Rng,
        M: RandomField + DivisionAlgebra<R> + Field + ComplexField<R> + std::fmt::Debug,
        R: RealField + FromPrimitive + ToPrimitive,
    {
        let n = G::matrix_dim();
        if n == 0 {
            return Err(LinkVariableError::InvalidDimension(n));
        }

        // Generate random matrix with entries in [-0.5, 0.5] via RandomField
        let mut data = Vec::with_capacity(n * n);
        for _ in 0..(n * n) {
            let val = M::generate_uniform(rng);
            data.push(val);
        }

        let tensor = CausalTensor::new(data, vec![n, n])
            .map_err(|e| LinkVariableError::TensorCreation(format!("{:?}", e)))?;

        let random_matrix = Self {
            data: tensor,
            _gauge: PhantomData,
            _scalar: PhantomData,
        };

        // Project to SU(N) to ensure unitarity and det = 1
        random_matrix.project_sun()
    }
}
