/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Morphism.lean`.

use deep_causality_haft::{FnMorphism, Morphism};

/// THEOREM_MAP: haft.morphism.identity
#[test]
fn test_morphism_identity() {
    // apply identity a = a
    let id_arrow = <FnMorphism as Morphism<FnMorphism>>::identity::<i32>();
    for x in [0, 42, -17] {
        assert_eq!(FnMorphism::apply(&id_arrow, x), x);
    }
}
