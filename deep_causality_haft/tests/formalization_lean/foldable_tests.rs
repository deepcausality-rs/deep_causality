/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Foldable.lean`.

use deep_causality_haft::{Foldable, OptionWitness, Pure};

/// THEOREM_MAP: haft.foldable.pure_compat
#[test]
fn test_foldable_pure_compat() {
    // fold (pure x) init f = f init x
    let f = |acc: i32, x: i32| acc + x * 10;
    assert_eq!(OptionWitness::fold(OptionWitness::pure(4), 2, f), f(2, 4));
    // and the unit case: fold none init f = init
    assert_eq!(OptionWitness::fold(None::<i32>, 2, f), 2);
}
