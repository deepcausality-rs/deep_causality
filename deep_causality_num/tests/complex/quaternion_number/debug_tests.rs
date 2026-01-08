/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Quaternion;

#[test]
fn test_debug_format() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let debug_str = format!("{:?}", q);
    assert_eq!(debug_str, "Quaternion { w: 1.0, x: 2.0, y: 3.0, z: 4.0 }");
}
