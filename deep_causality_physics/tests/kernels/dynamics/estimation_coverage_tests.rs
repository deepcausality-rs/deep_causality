/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage notes for the late dimension-mismatch guards in
//! `kalman_filter_linear_kernel` (estimation.rs:170-174 and 187-191).
//!
//! These two guards are *defensive* and provably unreachable for any input
//! that survives the earlier matmul chain:
//!
//! * Line 170-174 checks `x_pred.shape() != ky.shape()` where
//!   `ky = K · y`. `K` has shape `[n, m]` (from `P[n,n]·Hᵀ[n,m]·S⁻¹[m,m]`) and
//!   `y` matches the measurement column shape `[m, c]`, so `ky` is `[n, c]`.
//!   For `hx = H·x_pred` to have succeeded, `x_pred` must be `[n, c]` — exactly
//!   `ky`'s shape. The shapes therefore always agree once the matmuls succeed.
//!
//! * Line 187-191 checks `identity.shape() != kh.shape()` where
//!   `kh = K[n,m] · H[m,n] = [n, n]` and `identity` is built from
//!   `p_pred.shape()`, which `CausalTensor::identity` already required to be a
//!   square `[n, n]`. The shapes therefore always agree.
//!
//! Any attempt to violate either guard makes one of the preceding `matmul`
//! calls fail first (the tensor library rejects the inner-dimension mismatch),
//! so control never reaches the guard. The test below exercises the full happy
//! path (which flows *past* both guards via the non-error branch) and a
//! representative early-mismatch case, documenting the unreachable guards.

use deep_causality_physics::{PhysicsErrorEnum, kalman_filter_linear_kernel};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_kalman_full_path_flows_past_late_guards() {
    // A consistent 2-state / 1-measurement system. The run succeeds, which
    // means execution reached and passed both late guards via their
    // non-error branches.
    let x_pred = CausalTensor::new(vec![1.0, 2.0], vec![2, 1]).unwrap();
    let p_pred = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    let h = CausalTensor::new(vec![1.0, 0.0], vec![1, 2]).unwrap();
    let z = CausalTensor::new(vec![1.5], vec![1, 1]).unwrap();
    let r = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let q = CausalTensor::new(vec![0.0, 0.0, 0.0, 0.0], vec![2, 2]).unwrap();

    let res = kalman_filter_linear_kernel::<f64>(&x_pred, &p_pred, &z, &h, &r, &q);
    assert!(res.is_ok(), "expected success, got {res:?}");
    let (x_new, p_new) = res.unwrap();
    assert_eq!(x_new.shape(), &[2, 1]);
    assert_eq!(p_new.shape(), &[2, 2]);
}

#[test]
fn test_kalman_early_matmul_mismatch_preempts_late_guards() {
    // Force an inner-dimension mismatch in the very first `H · x_pred` matmul.
    // The kernel surfaces the failure long before the late guards, confirming
    // those guards cannot be the first failure point.
    let x_pred = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3, 1]).unwrap();
    let p_pred = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    // H is [1, 2]; H · x_pred[3,1] is an inner-dim mismatch (2 != 3).
    let h = CausalTensor::new(vec![1.0, 0.0], vec![1, 2]).unwrap();
    let z = CausalTensor::new(vec![1.5], vec![1, 1]).unwrap();
    let r = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let q = CausalTensor::new(vec![0.0, 0.0, 0.0, 0.0], vec![2, 2]).unwrap();

    let res = kalman_filter_linear_kernel::<f64>(&x_pred, &p_pred, &z, &h, &r, &q);
    assert!(res.is_err());
    // The error originates from the matmul layer, not the late shape guards.
    // A tensor-layer error wrapped as a non-DimensionMismatch variant is equally
    // fine; the point is that the late guards were not the failure site.
    if let PhysicsErrorEnum::DimensionMismatch(msg) = res.unwrap_err().0 {
        assert!(
            !msg.contains("State shape") && !msg.contains("Identity shape"),
            "unexpectedly reached a late guard: {msg}"
        );
    }
}
