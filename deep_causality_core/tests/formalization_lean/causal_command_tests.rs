/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witness for the single-hole functor laws of `CausalCommand`.
//!
//! Mirrors `lean/DeepCausalityFormal/Core/CausalCommand.lean`. The Lean proof shows
//! `CausalCommandWitness::fmap` maps the one sub-program hole (leaving the `target` index as
//! structure) and satisfies the functor laws `Free` requires of its operation functor; this witness
//! checks `fmap id = id` and `fmap (g∘f) = fmap g ∘ fmap f` on the real `CausalCommandWitness`.

use deep_causality_core::{CausalCommand, CausalCommandWitness};
use deep_causality_haft::Functor;

// ---- core.causal_command.functor_laws ----------------------------------------------------------

/// THEOREM_MAP: core.causal_command.functor_laws
#[test]
fn test_causal_command_functor_laws() {
    // Identity: fmap id = id — the `target` index and the hole are both preserved.
    let m = CausalCommand::RelayTo(3usize, 10i32);
    assert_eq!(
        CausalCommandWitness::fmap(m, |k: i32| k),
        CausalCommand::RelayTo(3usize, 10i32)
    );

    // Composition: fmap (g ∘ f) = fmap g ∘ fmap f.
    let f = |k: i32| k + 1;
    let g = |k: i32| k * 2;
    let lhs = CausalCommandWitness::fmap(CausalCommand::RelayTo(3usize, 10i32), |k| g(f(k)));
    let rhs = CausalCommandWitness::fmap(
        CausalCommandWitness::fmap(CausalCommand::RelayTo(3usize, 10i32), f),
        g,
    );
    assert_eq!(lhs, rhs);
    // The target index is untouched by the functor action.
    assert_eq!(lhs, CausalCommand::RelayTo(3usize, 22i32));
}
