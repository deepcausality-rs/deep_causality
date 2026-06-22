/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::StokesVector;

#[test]
fn test_stokes_vector_default() {
    let s: StokesVector<f64> = Default::default();
    assert!(s.inner().is_empty());
}
