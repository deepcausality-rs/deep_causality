/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MLX implementation of gamma matrix loader.
//!
//! Uploads gamma matrices to GPU memory once and caches the reference.

use super::BackendGamma;
use deep_causality_metric::Metric;
use deep_causality_tensor::{MlxBackend, TensorBackend, TensorData};

/// MLX implementation of BackendGamma.
///
/// This loader uploads gamma matrices to GPU memory on first access
/// and caches them for subsequent operations.
pub struct MlxGammaLoader;

impl<T> BackendGamma<MlxBackend, T> for MlxGammaLoader
where
    T: TensorData + Clone + std::ops::Neg<Output = T>,
{
    fn get_gammas(metric: &Metric) -> <MlxBackend as TensorBackend>::Tensor<T> {
        let n = metric.dimension();
        // Compute matrix_dim inline to avoid type inference issues
        let matrix_dim = 1usize << n.div_ceil(2);

        // Create gamma matrices on GPU
        // Shape: [n, matrix_dim, matrix_dim]
        let shape = [n, matrix_dim, matrix_dim];

        // Generate on CPU first, then upload to MLX
        // MLX's from_shape_fn will handle the upload
        MlxBackend::from_shape_fn(&shape, |idx| {
            let gamma_idx = idx[0];
            let row = idx[1];
            let col = idx[2];

            // Use same generation logic as CPU
            super::compute_gamma_element::<T>(gamma_idx, row, col, metric)
        })
    }

    fn get_basis_gammas(metric: &Metric) -> <MlxBackend as TensorBackend>::Tensor<T> {
        let n = metric.dimension();
        let matrix_dim = 1usize << n.div_ceil(2);
        let num_blades = 1 << n;

        let shape = [num_blades, matrix_dim, matrix_dim];

        // Compute on CPU then upload
        let mut data = Vec::with_capacity(num_blades * matrix_dim * matrix_dim);

        for i in 0..num_blades {
            let mut blade = vec![T::zero(); matrix_dim * matrix_dim];
            for k in 0..matrix_dim {
                blade[k * matrix_dim + k] = T::one();
            }

            for b in 0..n {
                if (i >> b) & 1 == 1 {
                    let mut next_blade = vec![T::zero(); matrix_dim * matrix_dim];
                    for r in 0..matrix_dim {
                        for c in 0..matrix_dim {
                            let mut sum = T::zero();
                            for k in 0..matrix_dim {
                                let val_a = blade[r * matrix_dim + k];
                                let val_b = super::compute_gamma_element::<T>(b, k, c, metric);
                                sum = sum + val_a * val_b;
                            }
                            next_blade[r * matrix_dim + c] = sum;
                        }
                    }
                    blade = next_blade;
                }
            }
            data.extend(blade);
        }

        super::from_data_helper::<MlxBackend, T>(&data, &shape)
    }

    fn get_dual_basis_gammas(metric: &Metric) -> <MlxBackend as TensorBackend>::Tensor<T> {
        let n = metric.dimension();
        let matrix_dim = 1usize << n.div_ceil(2);
        let num_blades = 1 << n;

        // Compute on CPU then upload
        let mut data = Vec::with_capacity(num_blades * matrix_dim * matrix_dim);

        for i in 0..num_blades {
            // 1. Reconstruct B
            let mut blade = vec![T::zero(); matrix_dim * matrix_dim];
            for k in 0..matrix_dim {
                blade[k * matrix_dim + k] = T::one();
            }

            for b in 0..n {
                if (i >> b) & 1 == 1 {
                    let mut next_blade = vec![T::zero(); matrix_dim * matrix_dim];
                    for r in 0..matrix_dim {
                        for c in 0..matrix_dim {
                            let mut sum = T::zero();
                            for k in 0..matrix_dim {
                                let val_a = blade[r * matrix_dim + k];
                                let val_b = super::compute_gamma_element::<T>(b, k, c, metric);
                                sum = sum + val_a * val_b;
                            }
                            next_blade[r * matrix_dim + c] = sum;
                        }
                    }
                    blade = next_blade;
                }
            }

            // 2. Compute square (0,0) element to find sign
            // (B * B)[0,0] = sum_k B[0,k] * B[k,0]
            let mut sq_00 = T::zero();
            for k in 0..matrix_dim {
                sq_00 = sq_00 + blade[k] * blade[k * matrix_dim];
            }

            // Inverse = blade * sq_00
            // Transpose

            let mut dual_blade = vec![T::zero(); matrix_dim * matrix_dim];
            for r in 0..matrix_dim {
                for c in 0..matrix_dim {
                    // Dual[r,c] = Inv[c,r] = (Blade[c,r] * sq_00)
                    dual_blade[r * matrix_dim + c] = blade[c * matrix_dim + r] * sq_00;
                }
            }
            data.extend(dual_blade);
        }

        let shape = [num_blades, matrix_dim, matrix_dim];
        super::from_data_helper::<MlxBackend, T>(&data, &shape)
    }
}
