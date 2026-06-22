/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::AbcdMatrix;

#[test]
fn test_abcd_matrix_default() {
    let m: AbcdMatrix<f64> = Default::default();
    assert!(m.inner().is_empty());
}
