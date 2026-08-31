/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::HKT2Unbound;

/// The explicit typed-arrow base: a family of arrows `P::Type<A, B>` (think `A → B`)
/// with an identity arrow and the ability to apply an arrow to an input.
///
/// # Category Theory
///
/// `Morphism` is the object/arrow interface of a category: every discovery operator
/// (and, later, every stage of the Causal Arrow) instances it. It is parameterized by a
/// two-argument HKT witness `P: HKT2Unbound`, so the concrete carrier of an arrow is the
/// witness's `Type<A, B>`.
///
/// # Why there is no `compose` here
///
/// General arrow composition `P::Type<A, B>` with `P::Type<B, C>` into `P::Type<A, C>`
/// over *capturing* closures has no single concrete carrier under the repo's
/// `unsafe_code = "forbid"` / no-`dyn` policy (closures are unnameable unique types, and
/// `Box<dyn Fn>` is a forbidden trait object). Identity plus application is the honest,
/// implementable base, and it is enough to host the iteration combinators on
/// [`Endomorphism`](crate::Endomorphism).
pub trait Morphism<P: HKT2Unbound> {
    /// The identity arrow `A → A`.
    fn identity<A>() -> P::Type<A, A>;

    /// Apply an arrow to an input, producing the output.
    fn apply<A, B>(arrow: &P::Type<A, B>, input: A) -> B;
}

/// The canonical function-pointer carrier for [`Morphism`]: an arrow `A → B` is a plain
/// `fn(A) -> B`.
///
/// Fully static, zero-capture, and free of `dyn`/trait objects. It covers any rule
/// expressible as a non-capturing function (for example the BRCD Meek pass, which reads
/// and writes only the graph). Rules that must capture configuration get a dedicated
/// carrier when a real call site needs one.
pub struct FnMorphism;

impl HKT2Unbound for FnMorphism {
    type Type<A, B> = fn(A) -> B;
}

impl Morphism<FnMorphism> for FnMorphism {
    #[inline]
    fn identity<A>() -> <FnMorphism as HKT2Unbound>::Type<A, A> {
        fn id<A>(a: A) -> A {
            a
        }
        id::<A>
    }

    #[inline]
    fn apply<A, B>(arrow: &<FnMorphism as HKT2Unbound>::Type<A, B>, input: A) -> B {
        (*arrow)(input)
    }
}
