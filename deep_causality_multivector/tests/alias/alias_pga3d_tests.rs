/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::PGA3DMultiVector;

#[test]
fn test_pga3d_new_point() {
    // Point (x, y, z) -> x*e032 + y*e013 + z*e021 + e123
    let p = PGA3DMultiVector::new_point(1.0, 2.0, 3.0);

    // Check metric
    assert_eq!(p.metric.dimension(), 4);

    // Check coefficients
    // e123 (14) -> 1.0
    assert_eq!(p.data[14], 1.0);
    // e032 (13) -> -1.0
    assert_eq!(p.data[13], -1.0);
    // e013 (11) -> 2.0
    assert_eq!(p.data[11], 2.0);
    // e021 (7) -> -3.0
    assert_eq!(p.data[7], -3.0);
}

#[test]
fn test_pga3d_translator() {
    // Translator T = 1 - 0.5(x*e01 + y*e02 + z*e03)
    let t = PGA3DMultiVector::translator(2.0, 4.0, 6.0);

    // Scalar (0) -> 1.0
    assert_eq!(t.data[0], 1.0);

    // e01 (3) -> -0.5 * 2.0 = -1.0
    assert_eq!(t.data[3], -1.0);

    // e02 (5) -> -0.5 * 4.0 = -2.0
    assert_eq!(t.data[5], -2.0);

    // e03 (9) -> -0.5 * 6.0 = -3.0
    assert_eq!(t.data[9], -3.0);
}
