/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// A map from one algebraic structure to another, carrying its **domain** and **codomain**.
///
/// # Why this exists
///
/// The `*Iso` traits in [`crate::iso`] are declared on a *pair* — `GroupIso<T>` requires
/// `Self: From<T>` and `T: From<Self>`, symmetric in the two ends. That shape can only ever express
/// an isomorphism, and because the two ends are interchangeable there is no map object and so
/// nothing to have a domain or a codomain.
///
/// The canonical embeddings that *define* the number tower are not isomorphisms. ℤ ↪ ℚ is injective
/// and not surjective — `1/2` is not in its image — so it can never be a `GroupIso`, and until now
/// it had nowhere to live: it exists as `Rational::from_integer`, an inherent constructor with no
/// statement that it preserves anything.
///
/// # Contract
///
/// A `Hom` is a function. Implementing it claims nothing beyond that; the structure it preserves is
/// claimed by the refinements, and the properties of the map by [`Injective`](crate::Injective) and
/// [`Surjective`](crate::Surjective).
///
/// # Not the only notion of morphism here
///
/// This models maps between **algebraic** structures — rings and fields. It is deliberately not the
/// morphism notion for other categories in this workspace: `deep_causality_haft` has `Category`,
/// `NaturalTransformation` and `Adjunction` for functors, and a quantum channel is a
/// completely-positive trace-preserving map that does *not* preserve multiplication, so it is not a
/// [`RingHom`](crate::RingHom) and never will be. Sharing the words "domain" and "codomain" does not
/// make them the same category.
pub trait Hom {
    /// The structure the map is defined on.
    type Domain;

    /// The structure the map lands in.
    type Codomain;

    /// Apply the map.
    fn apply(&self, x: Self::Domain) -> Self::Codomain;
}
