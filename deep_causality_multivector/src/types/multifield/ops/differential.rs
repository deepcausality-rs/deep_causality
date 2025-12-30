/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Differential operators for CausalMultiField.
//!
//! Implements curl, divergence, and gradient using central-difference stencils
//! via backend slicing operations.

use crate::CausalMultiField;
use deep_causality_num::Ring;
use deep_causality_tensor::{LinearAlgebraBackend, TensorData};

/// Axis enumeration for differential operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    fn index(self) -> usize {
        self as usize
    }
}

impl<B, T> CausalMultiField<B, T>
where
    B: LinearAlgebraBackend,
    T: TensorData + Clone,
{
    /// Computes the curl: ∇ × F.
    ///
    /// Returns the grade-2 component of the gradient ∇F.
    pub fn curl(&self) -> Self
    where
        T: Ring + Default + PartialOrd + std::ops::Div<Output = T>,
        B: crate::types::multifield::gamma::GammaProvider<T>,
    {
        // Curl is the bivector part (Grade 2) of the gradient
        self.gradient().grade_project(2)
    }

    /// Computes the divergence: ∇ · F.
    ///
    /// Returns the scalar part (Grade 0) of the gradient ∇F.
    pub fn divergence(&self) -> Self
    where
        T: Ring + Default + PartialOrd + std::ops::Div<Output = T>,
        B: crate::types::multifield::gamma::GammaProvider<T>,
    {
        // Divergence is the scalar part (Grade 0) of the gradient
        self.gradient().grade_project(0)
    }

    /// Computes the gradient: ∇F.
    ///
    /// Returns the full geometric derivative field (all grades).
    pub fn gradient(&self) -> Self
    where
        T: Ring + Default + PartialOrd + std::ops::Div<Output = T>,
        B: crate::types::multifield::gamma::GammaProvider<T>,
    {
        let dx = self.partial_derivative(Axis::X);
        let dy = self.partial_derivative(Axis::Y);
        let dz = self.partial_derivative(Axis::Z);

        self.construct_gradient(&dx, &dy, &dz)
    }

    /// Computes the partial derivative along a given axis.
    ///
    /// Uses central difference: ∂F/∂x ≈ (F[x+1] - F[x-1]) / (2dx)
    pub fn partial_derivative(&self, axis: Axis) -> B::Tensor<T>
    where
        T: Ring + Default + PartialOrd + std::ops::Div<Output = T>,
    {
        let idx = axis.index();
        let n = self.shape[idx];

        if n < 3 {
            // Cannot compute central difference with fewer than 3 points
            // Return zero tensor if not enough points
            return B::zeros(B::shape(&self.data).as_slice());
        }

        // Get the full tensor shape
        let shape = B::shape(&self.data);

        // Build slice ranges for forward and backward shifts
        let mut left_ranges = Vec::with_capacity(shape.len());
        let mut right_ranges = Vec::with_capacity(shape.len());

        // We assume 3D spatial dims are 0,1,2 for fields.
        for (i, &dim) in shape.iter().enumerate() {
            if i == idx {
                // Axis to differentiate: shift by 1
                left_ranges.push(0..dim - 2);
                right_ranges.push(2..dim);
            } else {
                left_ranges.push(0..dim);
                right_ranges.push(0..dim);
            }
        }

        // Slice to get shifted views
        // Note: backend slice logic usually takes range array.
        let left = B::slice(&self.data, &left_ranges);
        let right = B::slice(&self.data, &right_ranges);

        // Compute difference
        let diff = B::sub(&right, &left);

        // Manual Padding Logic (Backend-agnostic)
        // 1. Permute to move differentiated axis to 0
        let rank = shape.len();
        let mut perm_indices: Vec<usize> = (0..rank).collect();
        // swap idx and 0? No, remove idx and insert at 0.
        perm_indices.remove(idx);
        perm_indices.insert(0, idx);

        // permute diff
        let diff_perm = B::permute(&diff, &perm_indices);

        // 2. Download to vec
        let diff_vec = B::to_vec(&diff_perm);

        // 3. Pad
        // Current shape[idx] is N-2. Original is N.
        // Shape of permuted is [N-2, Rest...].
        // Block size = Rest... product.
        let block_size = diff_vec.len() / (shape[idx] - 2);

        let pad_block = vec![T::zero(); block_size];

        let mut padded_vec = Vec::with_capacity(diff_vec.len() + 2 * block_size);
        padded_vec.extend_from_slice(&pad_block);
        padded_vec.extend_from_slice(&diff_vec);
        padded_vec.extend_from_slice(&pad_block);

        // 4. Create new tensor [N, Rest...]
        let mut new_shape_perm = B::shape(&diff_perm);
        new_shape_perm[0] = shape[idx]; // Restore N

        let padded_perm = B::create_from_vec(padded_vec, &new_shape_perm);

        // 5. Permute back
        // Invert permutation.
        // Original: perm_indices maps New[i] -> Old[perm_indices[i]].
        // We want: Old[k] -> New[?].
        // We map 0 -> idx.
        // We moved idx to 0. Rest shifted.
        // Inverse permutation:
        let mut inv_perm = vec![0; rank];
        for (i, &p) in perm_indices.iter().enumerate() {
            inv_perm[p] = i;
        }

        let padded = B::permute(&padded_perm, &inv_perm);

        // Scale by 1/(2*dx)
        let two_dx = self.dx[idx] + self.dx[idx];
        let inv_two_dx = T::one() / two_dx;
        let scale_tensor = B::from_shape_fn(&[1], |_| inv_two_dx);

        B::mul(&padded, &scale_tensor)
    }

    /// Constructs gradient from partial derivatives.
    ///
    /// ∇F = ∑ γ_i ∂_i F
    fn construct_gradient(&self, dx: &B::Tensor<T>, dy: &B::Tensor<T>, dz: &B::Tensor<T>) -> Self
    where
        T: Ring + Default + PartialOrd,
        B: crate::types::multifield::gamma::GammaProvider<T>,
    {
        use crate::types::multifield::gamma::BackendGamma;

        let n = self.metric.dimension();
        let gammas = B::GammaLoader::get_gammas(&self.metric);
        let matrix_dim = Self::compute_matrix_dim(n);

        // We assume 3D spatial gradient (x, y, z) mapped to indices 0, 1, 2.

        // Helper for batched matmul: Gamma [D,D] * dx [Batch, D, D]
        let apply_gamma = |g: &B::Tensor<T>, t: &B::Tensor<T>| -> B::Tensor<T> {
            let shape_t = B::shape(t);
            // Assume t is [Nx, Ny, Nz, D, D]
            // Flatten spatial: [Batch, D, D]
            let batch_size = shape_t[0] * shape_t[1] * shape_t[2];

            let t_flat = B::reshape(t, &[batch_size, matrix_dim, matrix_dim]);

            // Permute [1, 0, 2] -> [D, Batch, D]
            let t_perm = B::permute(&t_flat, &[1, 0, 2]);

            // Reshape -> [D, Batch*D]
            let t_mat = B::reshape(&t_perm, &[matrix_dim, batch_size * matrix_dim]);

            // Matmul Gamma * T_mat -> [D, Batch*D]
            let res_mat = B::matmul(g, &t_mat);

            // Reshape back -> [D, Batch, D]
            let res_perm = B::reshape(&res_mat, &[matrix_dim, batch_size, matrix_dim]);

            // Permute back -> [Batch, D, D]
            let res_flat = B::permute(&res_perm, &[1, 0, 2]);

            // Define original shape for final result
            // [Nx, Ny, Nz, D, D]
            let orig_shape = [shape_t[0], shape_t[1], shape_t[2], matrix_dim, matrix_dim];
            B::reshape(&res_flat, &orig_shape)
        };

        // 1. Term X: dx * Gamma_0
        let g0_slice = B::slice(&gammas, &[0..1, 0..matrix_dim, 0..matrix_dim]);
        let g0 = B::reshape(&g0_slice, &[matrix_dim, matrix_dim]);
        let term_x = apply_gamma(&g0, dx);

        if n < 2 {
            return Self {
                data: term_x,
                metric: self.metric,
                dx: self.dx,
                shape: self.shape,
            };
        }

        // 2. Term Y: dy * Gamma_1
        let g1_slice = B::slice(&gammas, &[1..2, 0..matrix_dim, 0..matrix_dim]);
        let g1 = B::reshape(&g1_slice, &[matrix_dim, matrix_dim]);
        let term_y = apply_gamma(&g1, dy);

        let sum_xy = B::add(&term_x, &term_y);

        if n < 3 {
            return Self {
                data: sum_xy,
                metric: self.metric,
                dx: self.dx,
                shape: self.shape,
            };
        }

        // 3. Term Z: dz * Gamma_2
        let g2_slice = B::slice(&gammas, &[2..3, 0..matrix_dim, 0..matrix_dim]);
        let g2 = B::reshape(&g2_slice, &[matrix_dim, matrix_dim]);
        let term_z = apply_gamma(&g2, dz);

        let result = B::add(&sum_xy, &term_z);

        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}
