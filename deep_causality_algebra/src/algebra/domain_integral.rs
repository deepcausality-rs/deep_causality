/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CommutativeRing;

/// Represents an **Integral Domain**: a non-trivial commutative ring with no zero divisors.
///
/// This is the missing rung between [`CommutativeRing`](crate::CommutativeRing) and
/// [`EuclideanDomain`](crate::EuclideanDomain). A Euclidean domain is *by definition* an integral
/// domain carrying a Euclidean function, so the axioms were being promised anyway — they were just
/// bundled into the lower rung's documentation instead of standing on their own.
///
/// # Mathematical Definition
///
/// A commutative ring `R` is an integral domain when:
///
/// - **Non-triviality:** `1 ≠ 0`, so the ring has at least two elements.
/// - **No zero divisors:** `a · b = 0` implies `a = 0` or `b = 0`.
///
/// # What it buys: cancellation
///
/// The absence of zero divisors is exactly what licenses **cancellation** — from `a · b = a · c`
/// and `a ≠ 0` it follows that `b = c`, because `a · (b − c) = 0` forces `b − c = 0`. Cancellation
/// is what makes exact elimination over a ring well defined, and it is the property that
/// fraction-free (Bareiss) elimination rests on: each of its divisions is guaranteed exact by the
/// integral-domain structure, not by a Euclidean valuation.
///
/// Without this rung there is no way to bound an algorithm on "cancellation holds". Bounding on
/// [`CommutativeRing`](crate::CommutativeRing) silently admits types where it does not; bounding on
/// [`EuclideanDomain`](crate::EuclideanDomain) needlessly excludes ℚ, ℝ and ℂ, which are integral
/// domains but carry no meaningful Euclidean valuation in this tower.
///
/// # What it excludes, and why that matters here
///
/// `Dual<T>` is the case that makes this rung load-bearing rather than decorative. ℝ[ε] is a
/// commutative ring, but `ε · ε = 0` with `ε ≠ 0`, so ε is a zero divisor and cancellation fails.
/// `Dual<T>` is therefore a `CommutativeRing` and **not** an `IntegralDomain` — the same reason it
/// is not a `Field`.
///
/// The container types are excluded for the same reason: element-wise multiplication makes any
/// value with a zero entry a zero divisor, and matrix multiplication has zero divisors even over a
/// field. ℕ is excluded one rung lower — it has no additive inverses, so it is not a ring at all.
///
/// Non-commutative division rings are excluded by definition: `Quaternion<T>` has no zero divisors,
/// but an integral domain is commutative, so ℍ is a division ring rather than a domain.
///
/// # What implementing this promises
///
/// Both axioms above, neither of which the compiler can check. Like every law in this tower it is a
/// deliberate per-type assertion rather than something granted by a blanket — a blanket over
/// [`Field`](crate::Field) would be sound mathematically, since a field's inverses rule out zero
/// divisors, but it would hand the promise to any future type meeting the structural bound.
pub trait IntegralDomain: CommutativeRing {}

// ℤ. The motivating case: no zero divisors, which is what makes exact integer elimination valid.
impl IntegralDomain for i8 {}
impl IntegralDomain for i16 {}
impl IntegralDomain for i32 {}
impl IntegralDomain for i64 {}
impl IntegralDomain for i128 {}
impl IntegralDomain for isize {}

// ℝ, as modelled by the floats. A field has no zero divisors: from `a·b = 0` and `a ≠ 0`,
// `b = a⁻¹·a·b = a⁻¹·0 = 0`.
impl IntegralDomain for f32 {}
impl IntegralDomain for f64 {}
impl IntegralDomain for crate::Float106 {}

// 𝔽₂. A field has no zero divisors, and 1 ≠ 0 holds because the field has two elements. Stated
// per type rather than granted by a blanket over `Field`, so that a future field does not inherit
// the promise without asserting it.
impl IntegralDomain for deep_causality_num::Gf2 {}
