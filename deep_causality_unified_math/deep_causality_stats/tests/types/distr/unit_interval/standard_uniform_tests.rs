/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_stats::StandardUniform;

#[test]
fn test_derived_traits() {
    let s1 = StandardUniform;
    let s2 = s1;
    let s3 = s1; // Copy
    assert_eq!(format!("{:?}", s1), "StandardUniform");
    assert_eq!(format!("{:?}", s2), "StandardUniform");
    assert_eq!(format!("{:?}", s3), "StandardUniform");
}
