/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::hkt_unbound::HKT2Unbound;

/// The `Profunctor` trait represents a type constructor that is contravariant in its first argument
/// and covariant in its second argument.
///
/// # Category Theory
/// A **Profunctor** is a functor $P: \mathcal{C}^{op} \times \mathcal{D} \to \text{Set}$.
/// It can be thought of as a generalized function $A \to B$, where you can pre-process the input $A$
/// and post-process the output $B$.
///
/// *   **Dimap**: $(C \to A) \to (B \to D) \to P(A, B) \to P(C, D)$
///
/// # Mathematical Definition
/// Let $P$ be a profunctor. For morphisms $f: C \to A$ (pre-composition) and $g: B \to D$ (post-composition),
/// `dimap` yields a morphism $P(A, B) \to P(C, D)$.
///
/// # Use Cases
/// *   **Adapters**: Wrapping a core logic kernel with input decoders and output encoders.
/// *   **Optics**: Used heavily in Lens libraries to access and modify nested data structures.
/// *   **State Machines**: Transforming the input alphabet and output alphabet of a transducer.
pub trait Profunctor<P: HKT2Unbound> {
    /// Contravariant map on A (Input), Covariant map on B (Output).
    /// "Pre-process the input, Post-process the output."
    ///
    /// # Arguments
    /// * `pab`: The profunctor instance `P<A, B>`.
    /// * `f_pre`: The pre-processing function `C -> A`.
    /// * `f_post`: The post-processing function `B -> D`.
    fn dimap<A, B, C, D, F1, F2>(pab: P::Type<A, B>, f_pre: F1, f_post: F2) -> P::Type<C, D>
    where
        F1: FnMut(C) -> A + 'static,
        F2: FnMut(B) -> D + 'static,
        A: 'static,
        B: 'static,
        C: 'static,
        D: 'static;

    /// Map only the input (Contravariant).
    /// Equivalent to `dimap(f, id)`.
    fn lmap<A, B, C, F1>(pab: P::Type<A, B>, f_pre: F1) -> P::Type<C, B>
    where
        F1: FnMut(C) -> A + 'static,
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Self::dimap(pab, f_pre, |b| b)
    }

    /// Map only the output (Covariant).
    /// Equivalent to `dimap(id, g)`.
    fn rmap<A, B, D, F2>(pab: P::Type<A, B>, f_post: F2) -> P::Type<A, D>
    where
        F2: FnMut(B) -> D + 'static,
        A: 'static,
        B: 'static,
        D: 'static,
    {
        Self::dimap(pab, |a| a, f_post)
    }
}
