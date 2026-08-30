/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{AliasFunctor, OptionWitness, VecWitness};

#[test]
fn test_alias_functor_transform_option() {
    let val = Some(10);
    // transform alias fmap
    let res = OptionWitness::transform(val, |x| x * 2);
    assert_eq!(res, Some(20));
}

#[test]
fn test_alias_functor_transform_vec() {
    let val = vec![1, 2, 3];
    // transform alias fmap
    let res = VecWitness::transform(val, |x| x + 1);
    assert_eq!(res, vec![2, 3, 4]);
}
