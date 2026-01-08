/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;

#[test]
fn test_default_is_new() {
    let default_matrix: CsrMatrix<f64> = CsrMatrix::default();
    let new_matrix: CsrMatrix<f64> = CsrMatrix::new();
    assert_eq!(default_matrix.shape(), new_matrix.shape());
    assert!(default_matrix.values().is_empty());
    assert!(new_matrix.values().is_empty());
}
