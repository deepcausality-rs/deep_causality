/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Comonad.lean` (Uustalu–Vene 2008).

use deep_causality_haft::{BoxWitness, CoMonad};

/// THEOREM_MAP: haft.comonad.laws
#[test]
fn test_comonad_laws() {
    // The observations f = |b| **b * 2 and g = |b| **b + 100 are inlined at each use:
    // `extend`'s `FnMut(&F::Type<A>) -> B` bound fixes their parameter to `&Box<i32>`,
    // which a standalone `let` cannot name without tripping clippy::borrowed_box.
    let w = Box::new(11);

    // Left identity: extend extract = id
    assert_eq!(BoxWitness::extend(&w, BoxWitness::extract), w);

    // Right identity: extract ∘ extend f = f  (RHS is f applied to w directly)
    assert_eq!(
        BoxWitness::extract(&BoxWitness::extend(&w, |b| **b * 2)),
        *w * 2
    );

    // Associativity: extend g ∘ extend f = extend (g ∘ extend f)
    assert_eq!(
        BoxWitness::extend(&BoxWitness::extend(&w, |b| **b * 2), |b| **b + 100),
        BoxWitness::extend(&w, |w_prime| {
            *BoxWitness::extend(w_prime, |b| **b * 2) + 100
        })
    );
}
