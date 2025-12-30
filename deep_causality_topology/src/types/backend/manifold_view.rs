/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::RealField;
use deep_causality_tensor::LinearAlgebraBackend;
use deep_causality_tensor::TensorBackend;
use deep_causality_tensor::TensorData;

/// Accelerated Manifold Chart
pub struct ManifoldView<B: TensorBackend, T: TensorData> {
    /// Metric tensor field g_{mn}
    pub metric_tensor: <B as TensorBackend>::Tensor<T>,

    /// Precomputed inverse metric g^{mn}
    pub inverse_metric: <B as TensorBackend>::Tensor<T>,
}

impl<B: LinearAlgebraBackend, T: TensorData + RealField + From<f32> + core::iter::Sum>
    ManifoldView<B, T>
{
    pub fn new(metric: <B as TensorBackend>::Tensor<T>) -> Self {
        let inverse_metric = B::inverse(&metric);

        Self {
            metric_tensor: metric,
            inverse_metric,
        }
    }

    pub fn compute_christoffel(&self) -> <B as TensorBackend>::Tensor<T> {
        let mut shape = B::shape(&self.metric_tensor);
        let rank = shape.len();
        let n = shape[rank - 1]; // N

        let batch_rank = rank - 2;

        // For a metric tensor [..., N, N], the Christoffel symbols are [..., N, N, N].
        // If the metric is constant (Rank 2), derivative is zero.
        if batch_rank == 0 {
            shape.push(n);
            return B::zeros::<T>(&shape);
        }

        // Numerical Differentiation
        // Calculate partial derivatives \partial_k g_{ij}
        // Result shape: [..., N, N, N] where last dim is k (gradient direction)
        // We assume grid dimensions correspond to tensor indices 0..N-1
        // and boundary conditions are periodic (toroidal topology).
        let mut grads = Vec::with_capacity(n);
        let strides = B::strides(&self.metric_tensor);
        let total_size: usize = shape.iter().product();

        let half_val = T::from(0.5);
        let half = B::create(&[half_val], &[1]);

        // We iterate k from 0 to N-1 (gradient index)
        for k in 0..n {
            // Check if this coordinate dimension k corresponds to a grid dimension
            if k < batch_rank {
                // Perform central difference along dimension k
                // G(x + dx)
                let stride = strides[k];
                let g_plus = B::shifted_view(&self.metric_tensor, stride);

                // G(x - dx)
                // Shift in opposite direction
                let g_minus = B::shifted_view(&self.metric_tensor, total_size - stride);

                let diff = B::sub(&g_plus, &g_minus);

                // Grad = (Plus - Minus) * 0.5
                let grad = B::broadcast_op(&diff, &half, |a, b| Ok(a * b))
                    .expect("Gradient scaling failed");
                grads.push(grad);
            } else {
                // No grid dimension for this index, derivative is 0
                grads.push(B::zeros(&shape));
            }
        }

        // Stack to create D[..., i, j, k]
        // D_{ab,c} = \partial_c g_{ab}
        // Stack along new last axis.
        let d_tensor = B::stack(&grads, rank).expect("Stack failed");

        // Compute Lower Christoffel
        // \Gamma_{kij} = 0.5 * (\partial_i g_{kj} + \partial_j g_{ki} - \partial_k g_{ij})
        // Indices of D: (a, b, c) -> g_{ab, c}

        let idx_a = batch_rank;
        let idx_b = batch_rank + 1;
        let idx_c = batch_rank + 2;

        // Term 1: \partial_i g_{kj} -> need D(k, j, i) at (k, i, j) position.
        // Map: a->k (0), b->j (2), c->i (1).
        // Permute to (a, c, b) relative to block
        let mut perm_1: Vec<usize> = (0..batch_rank).collect();
        perm_1.push(idx_a);
        perm_1.push(idx_c);
        perm_1.push(idx_b);
        let term1 = B::permute(&d_tensor, &perm_1);

        // Term 2: \partial_j g_{ki} -> need D(k, i, j).
        // D(k, i, j) holds g_{ki, j} = \partial_j g_{ki}.
        // This is Identity. We use d_tensor directly.

        // Term 3: \partial_k g_{ij} -> need D(i, j, k).
        // D(i, j, k) holds g_{ij, k} = \partial_k g_{ij}.
        // Map: a->i (1), b->j (2), c->k (0).
        // Permute to (c, a, b) relative to block
        let mut perm_3: Vec<usize> = (0..batch_rank).collect();
        perm_3.push(idx_c);
        perm_3.push(idx_a);
        perm_3.push(idx_b);
        let term3 = B::permute(&d_tensor, &perm_3);

        // Sum = (T1 + T2 - T3) * 0.5
        // term2 is d_tensor (Identity permutation)
        let sum_12 = B::add(&term1, &d_tensor);
        let sub_3 = B::sub(&sum_12, &term3);

        let lower_gamma_raw =
            B::broadcast_op(&sub_3, &half, |a, b| Ok(a * b)).expect("Gamma scaling failed");

        // Contract with Inverse Metric
        // \Gamma^m_{ij} = I^{mk} \Gamma_{kij}
        // I: [..., m, k] (or N, N)
        // L: [..., k, i, j] (or N, N, N)

        // Flatten L to [..., k, (i*j)]
        let mut shape_flat_lower = shape.clone();
        shape_flat_lower.pop(); // pop N
        shape_flat_lower.push(n * n); // k * (i*j)? No.
        // We want (k, ij).
        // L is (k, i, j).
        // reshape to (..., k, i*j) works because memory layout of (i, j) is same as (i*j)

        let lower_flat = B::reshape(&lower_gamma_raw, &shape_flat_lower);

        // Matmul (N x N) * (N x N^2) -> (N x N^2)
        // [..., m, k] * [..., k, ij] -> [..., m, ij]
        let contracted = B::matmul(&self.inverse_metric, &lower_flat);

        // Final Reshape to [..., N, N, N]
        let mut final_shape = shape.clone();
        final_shape.push(n);

        B::reshape(&contracted, &final_shape)
    }
}
