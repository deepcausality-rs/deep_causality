/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! LinearAlgebraBackend implementation for MlxBackend.

use super::{MlxBackend, MlxTensor};
use crate::{LinearAlgebraBackend, TensorData};
use core::iter::Sum;
use deep_causality_num::{RealField, Ring};

impl LinearAlgebraBackend for MlxBackend {
    fn matmul<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + Ring + Default + PartialOrd,
    {
        let array =
            mlx_rs::ops::matmul(a.as_array(), b.as_array()).expect("MlxBackend::matmul: failed");
        MlxTensor::new(array)
    }

    fn qr<T>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>)
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        // MLX linalg::qr returns (Q, R)
        let (q, r) =
            mlx_rs::linalg::qr(input.as_array()).expect("MlxBackend::qr: decomposition failed");
        (MlxTensor::new(q), MlxTensor::new(r))
    }

    fn svd<T>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>, Self::Tensor<T>)
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        // MLX linalg::svd returns (U, S, Vt)
        let (u, s, vt) =
            mlx_rs::linalg::svd(input.as_array()).expect("MlxBackend::svd: decomposition failed");
        (MlxTensor::new(u), MlxTensor::new(s), MlxTensor::new(vt))
    }

    fn inverse<T>(input: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        // MLX linalg::inv is not supported on GPU yet.
        // We implement an explicit 4x4 inverse using basic ops for the common spacetime case.
        let shape = input.as_array().shape();
        let ndim = shape.len();
        if ndim >= 2 && shape[ndim - 1] == 4 && shape[ndim - 2] == 4 {
            match explicit_inverse_4x4(input.as_array()) {
                Ok(arr) => return MlxTensor::new(arr),
                Err(e) => eprintln!("MLX 4x4 inverse failed, falling back to CPU: {}", e),
            }
        }

        // Fallback or non-4x4
        use mlx_rs::StreamOrDevice;
        let array = mlx_rs::linalg::inv_device(input.as_array(), StreamOrDevice::cpu())
            .expect("MlxBackend::inverse: matrix singular or not square");
        MlxTensor::new(array)
    }

    fn cholesky_decomposition<T>(input: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        let array = mlx_rs::linalg::cholesky(input.as_array(), Some(false))
            .expect("MlxBackend::cholesky: decomposition failed");
        MlxTensor::new(array)
    }

    fn solve_least_squares_cholsky<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        let array = mlx_rs::linalg::solve(a.as_array(), b.as_array())
            .expect("MlxBackend::solve_least_squares_cholsky: solve failed");
        MlxTensor::new(array)
    }

    fn tensor_product<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + Ring + Default + PartialOrd,
    {
        // Generalized outer product
        let shape_a = a.as_array().shape();
        let shape_b = b.as_array().shape();

        let size_a: i32 = shape_a.iter().product();
        let size_b: i32 = shape_b.iter().product();

        // MLX reshape to column/row vectors
        let a_flat = a
            .as_array()
            .reshape(&[size_a, 1])
            .expect("MlxBackend::tensor_product: reshape a failed");
        let b_flat = b
            .as_array()
            .reshape(&[1, size_b])
            .expect("MlxBackend::tensor_product: reshape b failed");

        let res_flat = mlx_rs::ops::matmul(&a_flat, &b_flat)
            .expect("MlxBackend::tensor_product: matmul failed");

        // Construct new shape
        let mut new_shape = Vec::with_capacity(shape_a.len() + shape_b.len());
        new_shape.extend_from_slice(shape_a);
        new_shape.extend_from_slice(shape_b);

        let res = res_flat
            .reshape(&new_shape)
            .expect("MlxBackend::tensor_product: final reshape failed");

        MlxTensor::new(res)
    }
}

// Helper for explicit 4x4 inversion on GPU
fn explicit_inverse_4x4(input: &mlx_rs::Array) -> Result<mlx_rs::Array, String> {
    use mlx_rs::Array;
    use mlx_rs::ops::{add, concatenate, divide, multiply, reshape, split, subtract, transpose};

    let shape = input.shape();
    if shape.len() < 2 {
        return Err("Rank < 2".into());
    }

    // Flatten last 2 dims (4x4) into 16
    let mut flat_shape = shape.to_vec();
    flat_shape.pop();
    flat_shape.pop(); // Remove 4, 4

    // Reshape to [Batch..., 16].
    flat_shape.push(16);
    let flattened = reshape(input, &flat_shape).map_err(|e| e.to_string())?;

    // Split into 16 components
    let components = split(&flattened, 16, -1).map_err(|e| e.to_string())?;
    if components.len() != 16 {
        return Err("Split failed".into());
    }

    let c = |i: usize| &components[i];

    // Helpers
    let add_ = |a: &Array, b: &Array| add(a, b).unwrap();
    let sub_ = |a: &Array, b: &Array| subtract(a, b).unwrap();
    let mul_ = |a: &Array, b: &Array| multiply(a, b).unwrap();
    let div_ = |a: &Array, b: &Array| divide(a, b).unwrap();

    // Inverse 2x2 Helper: (a,b,c,d) -> (oa, ob, oc, od)
    let inv_2x2 = |a: &Array, b: &Array, c: &Array, d: &Array| -> (Array, Array, Array, Array) {
        let det = sub_(&mul_(a, d), &mul_(b, c));

        // Prevent inf/NaN on near-singular blocks with small epsilon
        // Create epsilon scalar and add via broadcast
        let eps_scalar = mlx_rs::Array::from_slice(&[1e-12f32], &[1]);
        let det_safe = add_(&det, &eps_scalar);

        let ones = mlx_rs::ops::ones_like(&det_safe).unwrap();
        let inv_det = div_(&ones, &det_safe);

        let zero = sub_(&det, &det);
        let neg_b = sub_(&zero, b);
        let neg_c = sub_(&zero, c);

        let oa = mul_(&inv_det, d);
        let ob = mul_(&inv_det, &neg_b);
        let oc = mul_(&inv_det, &neg_c);
        let od = mul_(&inv_det, a);
        (oa, ob, oc, od)
    };

    // MatMul 2x2 Helper
    let mul_2x2 = |a1: &Array,
                   b1: &Array,
                   c1: &Array,
                   d1: &Array,
                   a2: &Array,
                   b2: &Array,
                   c2: &Array,
                   d2: &Array| {
        let r1 = add_(&mul_(a1, a2), &mul_(b1, c2));
        let r2 = add_(&mul_(a1, b2), &mul_(b1, d2));
        let r3 = add_(&mul_(c1, a2), &mul_(d1, c2));
        let r4 = add_(&mul_(c1, b2), &mul_(d1, d2));
        (r1, r2, r3, r4)
    };

    let sub_blocks =
        |a1: &Array,
         b1: &Array,
         c1: &Array,
         d1: &Array,
         a2: &Array,
         b2: &Array,
         c2: &Array,
         d2: &Array| { (sub_(a1, a2), sub_(b1, b2), sub_(c1, c2), sub_(d1, d2)) };

    // Define Blocks
    // A: 0,1,4,5
    // B: 2,3,6,7
    // C: 8,9,12,13
    // D: 10,11,14,15

    // 1. InvA = A^-1
    let (ia0, ia1, ia2, ia3) = inv_2x2(c(0), c(1), c(4), c(5));

    // 2. T1 = C * InvA
    let (t1_0, t1_1, t1_2, t1_3) = mul_2x2(c(8), c(9), c(12), c(13), &ia0, &ia1, &ia2, &ia3);

    // 3. T2 = T1 * B
    let (t2_0, t2_1, t2_2, t2_3) = mul_2x2(&t1_0, &t1_1, &t1_2, &t1_3, c(2), c(3), c(6), c(7));

    // 4. S = D - T2
    let (s0, s1, s2, s3) = sub_blocks(c(10), c(11), c(14), c(15), &t2_0, &t2_1, &t2_2, &t2_3);

    // 5. InvS = S^-1
    let (is0, is1, is2, is3) = inv_2x2(&s0, &s1, &s2, &s3);

    // Negate InvS for use in other blocks
    let n_is0 = mlx_rs::ops::negative(&is0).map_err(|e| e.to_string())?;
    let n_is1 = mlx_rs::ops::negative(&is1).map_err(|e| e.to_string())?;
    let n_is2 = mlx_rs::ops::negative(&is2).map_err(|e| e.to_string())?;
    let n_is3 = mlx_rs::ops::negative(&is3).map_err(|e| e.to_string())?;
    // F21 = -InvS * T1.
    let (f21_0, f21_1, f21_2, f21_3) =
        mul_2x2(&n_is0, &n_is1, &n_is2, &n_is3, &t1_0, &t1_1, &t1_2, &t1_3);

    // F12 = -InvA * B * InvS.
    // Term T3 = InvA * B
    let (t3_0, t3_1, t3_2, t3_3) = mul_2x2(&ia0, &ia1, &ia2, &ia3, c(2), c(3), c(6), c(7));
    // F12 = T3 * (-InvS)
    let (f12_0, f12_1, f12_2, f12_3) =
        mul_2x2(&t3_0, &t3_1, &t3_2, &t3_3, &n_is0, &n_is1, &n_is2, &n_is3);

    // F11 = InvA - F12 * T1
    let (term_0, term_1, term_2, term_3) =
        mul_2x2(&f12_0, &f12_1, &f12_2, &f12_3, &t1_0, &t1_1, &t1_2, &t1_3);
    let (f11_0, f11_1, f11_2, f11_3) =
        sub_blocks(&ia0, &ia1, &ia2, &ia3, &term_0, &term_1, &term_2, &term_3);

    // Collect results
    let outputs = vec![
        &f11_0, &f11_1, &f12_0, &f12_1, &f11_2, &f11_3, &f12_2, &f12_3, &f21_0, &f21_1, &is0, &is1,
        &f21_2, &f21_3, &is2, &is3,
    ];

    // Reconstruct
    let concat = concatenate(&outputs).map_err(|e| e.to_string())?;

    // Reshape [16*Batch..., 1] -> [16, Batch...] (using -1)
    let dim16 = vec![16, -1];
    let inter = reshape(&concat, &dim16).map_err(|e| e.to_string())?; // [16, B]

    let transposed = transpose(&inter).map_err(|e| e.to_string())?; // [B, 16]

    let final_res = reshape(&transposed, shape).map_err(|e| e.to_string())?; // [B, 4, 4]
    Ok(final_res)
}
