/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::HKT;

use core::ops::{Add, Mul};
use deep_causality_num::Zero;

/// The `Adjunction` trait defines a pair of adjoint functors $L$ (Left) and $R$ (Right).
///
/// # Category Theory
/// An **Adjunction** $L \dashv R$ exists between two categories $\mathcal{C}$ and $\mathcal{D}$ if there is a
/// natural isomorphism between the set of morphisms:
/// $$ \text{Hom}_{\mathcal{D}}(L(A), B) \cong \text{Hom}_{\mathcal{C}}(A, R(B)) $$
///
/// This is one of the most profound concepts in mathematics, generalizing the idea of "opposites" or "duals".
/// Examples include Free/Forgetful functors, Currying/Uncurrying, and Quantifiers ($\exists \dashv \text{const} \dashv \forall$).
///
/// # Mathematical Definition
/// The isomorphism is defined by two natural transformations:
/// *   **Unit ($\eta$)**: $id \to R \circ L$
/// *   **Counit ($\epsilon$)**: $L \circ R \to id$
///
/// Satisfying the triangle identities:
/// 1.  $R(\epsilon) \circ \eta_R = id_R$
/// 2.  $\epsilon_L \circ L(\eta) = id_L$
///
/// # Use Cases
/// *   **Conservation Laws**: In Discrete Exterior Calculus (DEC), the Boundary Operator ($\partial$) and Exterior Derivative ($d$) are adjoints.
///     $\langle d\phi, J \rangle = \langle \phi, \partial J \rangle$.
/// *   **Optimization**: Relating a constraint space (Primal) to a Lagrange multiplier space (Dual).
pub trait Adjunction<L, R>
where
    L: HKT,
    R: HKT,
{
    /// The Left Adjunct: $(L(A) \to B) \to (A \to R(B))$
    /// Transforms a function on the "Left" structure to a function on the "Right" structure.
    fn left_adjunct<A, B, F>(a: A, f: F) -> R::Type<B>
    where
        F: Fn(L::Type<A>) -> B;

    /// The Right Adjunct: $(A \to R(B)) \to (L(A) \to B)$
    /// Transforms a function on the "Right" structure to a function on the "Left" structure.
    fn right_adjunct<A, B, F>(la: L::Type<A>, f: F) -> B
    where
        F: Fn(A) -> R::Type<B>;

    /// The Unit of the Adjunction: $A \to R(L(A))$
    /// Embeds a value into the Right-Left context.
    fn unit<A>(a: A) -> R::Type<L::Type<A>>;

    /// The Counit of the Adjunction: $L(R(B)) \to B$
    /// Collapses the Left-Right context back to a value.
    fn counit<B>(lrb: L::Type<R::Type<B>>) -> B;
}

/// The `BoundedAdjunction` trait defines a pair of adjoint functors $L$ (Left) and $R$ (Right)
/// that require a runtime `Context` to operate.
///
/// This is useful when the functors depend on an environment (like a Metric, Shape, or Topology)
/// that cannot be fully captured in the static type system.
pub trait BoundedAdjunction<L, R, Context>
where
    L: HKT,
    R: HKT,
{
    /// The Left Adjunct: $(L(A) \to B) \to (A \to R(B))$
    /// Transforms a function on the "Left" structure to a function on the "Right" structure,
    /// using the provided context.
    fn left_adjunct<A, B, F>(ctx: &Context, a: A, f: F) -> R::Type<B>
    where
        F: Fn(L::Type<A>) -> B,
        A: Clone + Zero + Copy + PartialEq,
        B: Clone;

    /// The Right Adjunct: $(A \to R(B)) \to (L(A) \to B)$
    /// Transforms a function on the "Right" structure to a function on the "Left" structure,
    /// using the provided context.
    fn right_adjunct<A, B, F>(ctx: &Context, la: L::Type<A>, f: F) -> B
    where
        F: FnMut(A) -> R::Type<B>,
        A: Clone + Zero,
        B: Clone + Zero + Add<Output = B> + Mul<Output = B>;

    /// The Unit of the Adjunction: $A \to R(L(A))$
    /// Embeds a value into the Right-Left context, using the provided context.
    fn unit<A>(ctx: &Context, a: A) -> R::Type<L::Type<A>>
    where
        A: Clone + Zero + Copy + PartialEq;

    /// The Counit of the Adjunction: $L(R(B)) \to B$
    /// Collapses the Left-Right context back to a value, using the provided context.
    fn counit<B>(ctx: &Context, lrb: L::Type<R::Type<B>>) -> B
    where
        B: Clone + Zero + Add<Output = B> + Mul<Output = B>;
}
