/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Gf2;

#[test]
fn test_display_renders_digits_not_booleans() {
    assert_eq!(format!("{}", Gf2::ZERO), "0");
    assert_eq!(format!("{}", Gf2::ONE), "1");
}
