/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Hom, Injective, RingHom, Surjective};

/// The composite `g ∘ f`: apply `f`, then `g`.
///
/// The domain is `F`'s and the codomain is `G`'s, and the two are required to meet — `G::Domain`
/// must be `F::Codomain`, so a mismatched composite does not typecheck. That constraint is the
/// reason a map needs named ends in the first place.
///
/// # Composition preserves each property
///
/// Each of the impls below is a **theorem**, not a fresh promise, which is why they are blanket
/// implementations rather than per-map assertions:
///
/// - the composite of two ring homomorphisms preserves `+`, `·` and `1`, since each does;
/// - the composite of two injections is injective;
/// - the composite of two surjections is surjective.
///
/// [`Bijective`](crate::Bijective) follows from the last two without an impl of its own.
///
/// This is what lets the number tower be written as a chain, ℕ ↪ ℤ ↪ ℚ ↪ ℝ ↪ ℂ, with the composite
/// carrying its labels rather than losing them at each step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Compose<F, G> {
    /// Applied first.
    pub f: F,
    /// Applied second.
    pub g: G,
}

impl<F, G> Compose<F, G> {
    /// Build `g ∘ f`.
    pub const fn new(f: F, g: G) -> Self {
        Self { f, g }
    }
}

impl<F, G> Hom for Compose<F, G>
where
    F: Hom,
    G: Hom<Domain = F::Codomain>,
{
    type Domain = F::Domain;
    type Codomain = G::Codomain;

    fn apply(&self, x: Self::Domain) -> Self::Codomain {
        self.g.apply(self.f.apply(x))
    }
}

// Theorems, not promises: each property is closed under composition.
impl<F, G> RingHom for Compose<F, G>
where
    F: RingHom,
    G: RingHom<Domain = F::Codomain>,
{
}

impl<F, G> Injective for Compose<F, G>
where
    F: Injective,
    G: Injective<Domain = F::Codomain>,
{
}

impl<F, G> Surjective for Compose<F, G>
where
    F: Surjective,
    G: Surjective<Domain = F::Codomain>,
{
}
