/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_cache::family_key;

#[test]
fn family_key_sorts_and_dedups_parents() {
    assert_eq!(family_key(2, &[3, 1, 1]), (2, vec![1, 3]));
    assert_eq!(family_key(0, &[]), (0, vec![]));
    // Order-independent.
    assert_eq!(family_key(0, &[1, 2]), family_key(0, &[2, 1]));
}
