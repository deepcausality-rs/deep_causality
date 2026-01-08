/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Quaternion;

#[test]
fn test_neg() {
    let q = Quaternion::new(1.0, -2.0, 3.0, -4.0);
    let expected = Quaternion::new(-1.0, 2.0, -3.0, 4.0);
    assert_eq!(-q, expected);
}
