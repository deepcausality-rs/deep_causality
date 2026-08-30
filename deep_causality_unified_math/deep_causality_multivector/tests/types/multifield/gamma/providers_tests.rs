/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for gamma matrix functions.
//!
//! NOTE: The backend-specific GammaProvider tests have been removed as the
//! backend abstraction layer has been simplified. The gamma functions now
//! return CausalTensor<T> directly. See cpu_tests.rs for the updated tests.

use deep_causality_metric::Metric;
use deep_causality_multivector::{get_basis_gammas, get_dual_basis_gammas, get_gammas};

#[test]
fn test_gammas_consistency_cl2() {
    let metric = Metric::from_signature(2, 0, 0);

    // All gamma functions should work with f32 and f64
    let gammas_f32 = get_gammas::<f32>(&metric);
    let gammas_f64 = get_gammas::<f64>(&metric);

    assert_eq!(gammas_f32.shape(), gammas_f64.shape());
}

#[test]
fn test_basis_gammas_consistency_cl2() {
    let metric = Metric::from_signature(2, 0, 0);

    let basis_f32 = get_basis_gammas::<f32>(&metric);
    let basis_f64 = get_basis_gammas::<f64>(&metric);

    assert_eq!(basis_f32.shape(), basis_f64.shape());
}

#[test]
fn test_dual_basis_gammas_consistency_cl2() {
    let metric = Metric::from_signature(2, 0, 0);

    let dual_f32 = get_dual_basis_gammas::<f32>(&metric);
    let dual_f64 = get_dual_basis_gammas::<f64>(&metric);

    assert_eq!(dual_f32.shape(), dual_f64.shape());
}
