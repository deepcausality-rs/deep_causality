/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{AliasFoldable, VecWitness};

#[test]
fn test_alias_foldable_reduce() {
    let vec = vec![1, 2, 3, 4];
    // reduce alias fold
    let sum = VecWitness::reduce(vec, 0, |acc, x| acc + x);
    assert_eq!(sum, 10);
}
