/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Conversions between CausalMultiField and other representations.
//!
//! - `from_coefficients`: Upload from CPU CausalMultiVector collection
//! - `to_coefficients`: Download to CPU CausalMultiVector collection
//! - Factory methods: `zeros`, `ones`

use crate::CausalMultiField;
use crate::CausalMultiVector;
use crate::types::multifield::gamma::BackendGamma;
use deep_causality_metric::Metric;
use deep_causality_tensor::{LinearAlgebraBackend, TensorData};

impl<B, T> CausalMultiField<B, T>
where
    B: LinearAlgebraBackend,
    T: TensorData,
{
    /// Creates a field filled with zero multivectors.
    ///
    /// # Arguments
    /// * `shape` - Grid dimensions [Nx, Ny, Nz]
    /// * `metric` - The metric signature of the algebra
    /// * `dx` - Grid spacing [dx, dy, dz]
    pub fn zeros(shape: [usize; 3], metric: Metric, dx: [T; 3]) -> Self
    where
        T: Clone,
    {
        let matrix_dim = Self::compute_matrix_dim(metric.dimension());
        let full_shape = [shape[0], shape[1], shape[2], matrix_dim, matrix_dim];
        let data = B::zeros(&full_shape);

        Self {
            data,
            metric,
            dx,
            shape,
        }
    }

    /// Creates a field filled with identity matrices (scalar 1).
    ///
    /// Each cell contains the identity element of the algebra.
    pub fn ones(shape: [usize; 3], metric: Metric, dx: [T; 3]) -> Self
    where
        T: Clone,
    {
        let matrix_dim = Self::compute_matrix_dim(metric.dimension());
        let full_shape = [shape[0], shape[1], shape[2], matrix_dim, matrix_dim];

        // Create identity matrices for each cell
        let data = B::from_shape_fn(&full_shape, |idx| {
            // Identity matrix: 1 on diagonal, 0 elsewhere
            if idx[3] == idx[4] {
                T::one()
            } else {
                T::zero()
            }
        });

        Self {
            data,
            metric,
            dx,
            shape,
        }
    }

    /// Creates a field from a collection of CausalMultiVectors.
    ///
    /// The multivectors are converted to Matrix Representation and uploaded to the backend.
    /// Uses vectorized operations for efficiency.
    ///
    /// # Arguments
    /// * `mvs` - Flat array of multivectors in row-major order
    /// * `shape` - Grid dimensions [Nx, Ny, Nz]
    /// * `dx` - Grid spacing [dx, dy, dz]
    pub fn from_coefficients(mvs: &[CausalMultiVector<T>], shape: [usize; 3], dx: [T; 3]) -> Self
    where
        T: Clone + Default + PartialOrd,
        B: crate::types::multifield::gamma::GammaProvider<T>,
    {
        let expected_len = shape[0] * shape[1] * shape[2];
        assert_eq!(
            mvs.len(),
            expected_len,
            "Expected {} multivectors, got {}",
            expected_len,
            mvs.len()
        );

        if mvs.is_empty() {
            panic!("Cannot create field from empty multivector list");
        }

        let metric = mvs[0].metric();
        let matrix_dim = Self::compute_matrix_dim(metric.dimension());

        // 1. Upload coefficients to backend tensor [TotalCells, NumBlades]
        let num_blades = 1 << metric.dimension();

        // Flatten all multivector data into a single vector
        let mut flat_data = Vec::with_capacity(mvs.len() * num_blades);
        for mv in mvs {
            flat_data.extend_from_slice(&mv.data);
        }

        let coeffs_shape = [mvs.len(), num_blades];
        let coeffs_tensor =
            crate::types::multifield::gamma::from_data_helper::<B, T>(&flat_data, &coeffs_shape);

        // 2. Generate all basis matrices Γ_I on backend [NumBlades, MatrixDim, MatrixDim]
        // reshaping to [NumBlades, MatrixDim * MatrixDim] for matmul
        let basis_tensor = B::GammaLoader::get_basis_gammas(&metric);
        let basis_flat = B::reshape(&basis_tensor, &[num_blades, matrix_dim * matrix_dim]);

        // 3. Compute M = Σ c_I Γ_I via Matrix Multiplication
        // [Batch, Blades] @ [Blades, D*D] -> [Batch, D*D]
        let matrices_flat = B::matmul(&coeffs_tensor, &basis_flat);

        // 4. Reshape to [Nx, Ny, Nz, D, D]
        let full_shape = [shape[0], shape[1], shape[2], matrix_dim, matrix_dim];
        let data = B::reshape(&matrices_flat, &full_shape);

        Self {
            data,
            metric,
            dx,
            shape,
        }
    }

    /// Downloads the field to a collection of CausalMultiVectors.
    ///
    /// Converts from Matrix Representation back to coefficient form using trace projection.
    /// c_I = (1/d) * Tr(M * Γ_I⁻¹)
    pub fn to_coefficients(&self) -> Vec<CausalMultiVector<T>>
    where
        T: Clone + Default + PartialOrd + std::ops::Div<Output = T>, // Removed FromPrimitive
        B: crate::types::multifield::gamma::GammaProvider<T>,
    {
        use crate::types::multifield::gamma::BackendGamma;

        let num_cells = self.num_cells();
        let num_blades = 1 << self.metric.dimension();
        let matrix_dim = Self::compute_matrix_dim(self.metric.dimension());

        // Convert matrix_dim (which is 2^k) to T using Ring arithmetic
        // matrix_dim = 2^exponent
        let exponent = self.metric.dimension().div_ceil(2);
        let mut d_val = T::one();
        let two = T::one() + T::one();

        for _ in 0..exponent {
            d_val = d_val * two;
        }

        let scale = T::one() / d_val;

        // 1. Flatten field data to [Batch, D*D]
        let field_flat = B::reshape(&self.data, &[num_cells, matrix_dim * matrix_dim]);

        // 2. Generate inverse basis matrices (Γ_I)⁻¹
        // We use (Γ_I)⁻¹^T for the dot product projection form: Tr(A B) = vec(A) . vec(B^T)
        let basis_dual_tensor = B::GammaLoader::get_dual_basis_gammas(&self.metric); // [NumBlades, D, D]

        // We need [D*D, NumBlades] for the multiplication M @ Basis
        // Reshape basis duals to [NumBlades, D*D]
        let basis_dual_flat =
            B::reshape(&basis_dual_tensor, &[num_blades, matrix_dim * matrix_dim]);

        // Transpose to [D*D, NumBlades]
        // Use permute if transpose is not available
        let basis_dual_t = B::permute(&basis_dual_flat, &[1, 0]);

        // 3. Project: Coeffs = Field @ BasisDual_T
        let coeffs_raw = B::matmul(&field_flat, &basis_dual_t);

        // 4. Scale by 1/d
        let scale_tensor = B::from_shape_fn(&[1], |_| scale);
        let coeffs = B::mul(&coeffs_raw, &scale_tensor);

        // 5. Download and chunk into Multivectors
        let flat_coeffs = B::to_vec(&coeffs);

        flat_coeffs
            .chunks(num_blades)
            .map(|chunk| CausalMultiVector::unchecked(chunk.to_vec(), self.metric))
            .collect()
    }

    /// Computes the matrix dimension for a given algebra dimension.
    ///
    /// For Cl(p,q,r), the matrix dimension is 2^⌈N/2⌉ where N = p + q + r.
    pub fn compute_matrix_dim(n: usize) -> usize {
        1 << n.div_ceil(2)
    }
}
