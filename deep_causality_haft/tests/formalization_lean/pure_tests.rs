/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Pure.lean` (pointed functor; Mac Lane §I.4).

use deep_causality_haft::{Functor, OptionWitness, Pure};

/// THEOREM_MAP: haft.pure.naturality
#[test]
fn test_pure_naturality() {
    // fmap f ∘ pure = pure ∘ f — the naturality square of η.
    let f = |a: i32| a * 3;
    assert_eq!(
        OptionWitness::fmap(OptionWitness::pure(14), f),
        OptionWitness::pure(f(14))
    );
}
