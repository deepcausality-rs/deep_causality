/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{HKT3Unbound, Satisfies};

/// The `MonoidalMerge` trait models the "fusion" or "interaction" of two contexts to produce
/// a third.
///
/// # Category Theory
/// Restricted to the diagonal $D(A) = P\langle A, A, A\rangle$, `merge` is `liftA2`, the fused
/// form of the structure map
/// $$ \varphi : D(A) \otimes D(B) \to D(A \otimes B) $$
/// (McBride & Paterson 2008 §7; related to Day convolution). The two are interdefinable:
/// `merge(pa, pb, f) == fmap(zip(pa, pb), |(a, b)| f(a, b))` and
/// `zip(pa, pb) == merge(pa, pb, |a, b| (a, b))`.
///
/// This is a **semigroupal** structure, not a lax monoidal one. A lax monoidal functor is a
/// triple $(F, \varphi, \eta)$; this trait carries $\varphi$ alone. There is no unit
/// $\eta : I \to D(I)$ anywhere in it and none is derivable from `merge`, so the unitality
/// coherence conditions cannot be stated against this trait, only associativity and naturality.
/// An earlier version of this docstring called it lax monoidal, which overclaimed by exactly the
/// unit. See `openspec/notes/archive/hkt_gat/monoidal-applicative.md` §4.1 for the split between the
/// $\varphi$-only structure and the full $(\varphi, \eta)$ one.
///
/// # Naming history
/// This trait was previously named `Promonad`. In the categorical literature a *promonad* is
/// a monad in the bicategory of profunctors — equivalently an identity-on-objects functor
/// (Loregian, *(Co)end Calculus*, §5.2; Jacobs, Heunen & Hasuo, *Categorical semantics for
/// arrows*, JFP 19, 2009) — which this trait is not. It was renamed to say what it is; the
/// former `fuse` operation (whose result type `C` was structurally undetermined — no lawful
/// implementation existed for value-carrying carriers) was removed in the same change. See
/// `openspec/notes/archive/causal-algebra/haft-formalization-deviations.md`, D3/P-1.
///
/// # Law (Informal)
///
/// **Binaturality of `merge`**: `merge(fmap(pa, p), fmap(pb, q), h) ==
/// merge(pa, pb, |x, y| h(p(x), q(y)))` — mapping the inputs first equals merging with the
/// composed combiner. Machine-checked in `lean/DeepCausalityFormal/Haft/MonoidalMerge.lean`.
/// Laws are stated for pure functions; a stateful `FnMut` closure voids them.
///
/// # Use Cases
/// *   **Tensor Contraction**: Merging a Vector $u$ and a Dual Vector $v^*$ to produce a Scalar.
/// *   **Quantum Entanglement**: Combining Qubit A and Qubit B into an Entangled Pair C.
/// *   **Force Calculation**: Combining Current $J$ and Magnetic Field $B$ to produce Force $F$.
pub trait MonoidalMerge<P: HKT3Unbound> {
    /// Merges two contexts into a third.
    ///
    /// # Arguments
    /// * `pa`: The first context (Input A).
    /// * `pb`: The second context (Input B).
    /// * `f`: A function to combine the inner values $A$ and $B$ into $C$.
    fn merge<A, B, C, F>(
        pa: P::Type<A, A, A>, // Simplified signature for demonstration
        pb: P::Type<B, B, B>,
        f: F,
    ) -> P::Type<C, C, C>
    where
        A: Satisfies<P::Constraint>,
        B: Satisfies<P::Constraint>,
        C: Satisfies<P::Constraint>,
        F: FnMut(A, B) -> C;
}
