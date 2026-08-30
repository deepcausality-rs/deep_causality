/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::algebra::operator::Additive;
use crate::{AddMonoid, Commutative};
use crate::{Annihilating, Distributive, MulMonoid};

/// Represents a **Semiring** in abstract algebra.
///
/// A semiring is a `Ring` without additive inverses. It is the structure the natural numbers ℕ
/// have and cannot exceed: `3 - 5` has no value in ℕ, so there is no `-a`, and the additive
/// monoid never becomes a group.
///
/// # Mathematical Definition
///
/// A set `R` with two operations is a semiring if it satisfies:
///
/// 1.  **Under Addition:** `R` forms a commutative `AddMonoid`.
///     - Addition is associative: `(a + b) + c = a + (b + c)`
///     - Addition is commutative: `a + b = b + a`
///     - There is an additive identity `0`: `a + 0 = a`
///     - There is **no** requirement of additive inverses. This is the whole difference.
///
/// 2.  **Under Multiplication:** `R` forms a `MulMonoid`.
///     - Multiplication is associative: `(a * b) * c = a * (b * c)`
///     - There is a multiplicative identity `1`: `a * 1 = a`
///
/// 3.  **Distributivity:** `a * (b + c) = a * b + a * c` and `(a + b) * c = a * c + b * c`
///
/// 4.  **Annihilation:** `0 * a = a * 0 = 0`, recorded by [`Annihilating`](crate::Annihilating)
///
/// # Annihilation is an axiom here, not a theorem
///
/// This is the subtle part, and the reason a semiring is not merely "a ring with a bound
/// removed". In a `Ring` the annihilation law is *derived*:
///
/// ```text
/// 0·a = (0 + 0)·a = 0·a + 0·a        by the additive identity and distributivity
///   0 = 0·a                          by adding −(0·a) to both sides
/// ```
///
/// That last step consumes an additive inverse. A semiring has none, so the derivation is
/// unavailable and the law must be assumed. Implementing this trait therefore promises
/// annihilation independently. That is why [`Annihilating`](crate::Annihilating) is a supertrait
/// here and not merely a line of prose: a `Semiring` bound would otherwise assert a law that no
/// implementor had promised.
///
/// # Relationship to `Ring`
///
/// Every ring is a semiring, and the bounds here are deliberately a strict subset of the ones
/// `Ring` requires, so every `Ring` really is a `Semiring`.
///
/// Getting that subset right needed care. [`AddMonoid`](crate::AddMonoid) once required
/// `AddAssign`, which `Ring`'s additive side never has, and while it did this trait was
/// accidentally *stronger* than `Ring`: `CausalTensor` implements `AddAssign<T>` for a scalar
/// right-hand side and never `AddAssign<Self>`, so it satisfied `Ring` while failing `Semiring` —
/// the exact inversion of what a weakening should do. `AddMonoid` is now
/// `Add + Clone + Associative<Additive> + Zero`, which together with `Commutative<Additive>` is
/// [`AbelianGroup`](crate::AbelianGroup) with `Sub` and `Neg` dropped. That is the additive side
/// of a semiring, and it is what the bound below states.
///
/// `Ring` is nevertheless not declared as `Ring: Semiring`. Re-rooting the supertrait changes
/// nothing about membership, since the blanket impls already give every `Ring` the weaker bound,
/// and the tower is left flat rather than churned for a relationship the type system already
/// enforces.
///
/// ## Examples
/// - Natural numbers ℕ (`u8`–`u128`, `usize`) — the motivating case
/// - Every `Ring`, and therefore every `Field`
///
/// ## Counter-examples
/// - Any structure without a multiplicative identity, which is a *rig* rather than a semiring
pub trait Semiring:
    AddMonoid + Commutative<Additive> + MulMonoid + Distributive + Annihilating
{
}
impl<T> Semiring for T where
    T: AddMonoid + Commutative<Additive> + MulMonoid + Distributive + Annihilating
{
}
