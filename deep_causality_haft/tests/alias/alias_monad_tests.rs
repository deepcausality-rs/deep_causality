/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{AliasMonad, OptionWitness, VecWitness};

#[test]
fn test_alias_monad_chain() {
    let val = Some(10);
    // chain alias bind
    let res = OptionWitness::chain(val, |x| Some(x * 2));
    assert_eq!(res, Some(20));
}

#[test]
fn test_alias_monad_flatten() {
    let nested = Some(Some(42));
    // flatten alias join
    let res = OptionWitness::flatten(nested);
    assert_eq!(res, Some(42));
}

#[test]
fn test_alias_monad_vec_chain() {
    let val = vec![1, 2];
    // chain alias bind (flatMap)
    let res = VecWitness::chain(val, |x| vec![x, x * 10]);
    assert_eq!(res, vec![1, 10, 2, 20]);
}
