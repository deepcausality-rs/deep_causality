/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AddMonoid, Distributive, MulMonoid};

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
/// 4.  **Annihilation:** `0 * a = a * 0 = 0`
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
/// annihilation independently — the compiler cannot check it, and neither can the other axioms.
///
/// # Relationship to `Ring`
///
/// Every ring is a semiring, and every type in this crate that satisfies `Ring` also satisfies
/// `Semiring`, because the bounds here are a subset of the ones `Ring` requires. That membership
/// is what matters and it is correct. `Ring` is nevertheless **not** declared as
/// `Ring: Semiring`: re-rooting it that way evicts `CausalTensor` and `CausalTensorTrain`, which
/// implement `AddAssign<T>` for a scalar right-hand side but never `AddAssign<Self>`, and so miss
/// the `AddMonoid` bound. The hierarchy is left flat rather than made faithful at the cost of two
/// production types silently losing `Ring`.
///
/// ## Examples
/// - Natural numbers ℕ (`u8`–`u128`, `usize`) — the motivating case
/// - Every `Ring`, and therefore every `Field`
///
/// ## Counter-examples
/// - Any structure without a multiplicative identity, which is a *rig* rather than a semiring
pub trait Semiring: AddMonoid + MulMonoid + Distributive {}
impl<T> Semiring for T where T: AddMonoid + MulMonoid + Distributive {}
