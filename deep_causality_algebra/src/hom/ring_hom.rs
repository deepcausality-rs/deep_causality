/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Hom;

/// A [`Hom`] that preserves the ring structure.
///
/// # Laws
///
/// For all `a, b` in the domain:
///
/// - `f(a + b) == f(a) + f(b)`
/// - `f(a · b) == f(a) · f(b)`
/// - `f(1) == 1`  — **unital**, and not implied by the other two
///
/// Preservation of `0` and of negation follow from additivity, so they are not stated separately.
/// Unitality does not follow: the zero map preserves both operations and sends `1` to `0`.
///
/// Like every law in this tower these are unverifiable promises, asserted per map.
///
/// # What it excludes, and why that is the point
///
/// The cast layer in `deep_causality_num` presents widening and truncation with the same shape —
/// `FromPrimitive` and `ToPrimitive` — so nothing distinguishes a structure-preserving conversion
/// from a structure-destroying one. `i64 → f64` is a ring homomorphism below `2^53`. `f64 → i64`
/// is not one at all: it truncates, and `f(0.5 + 0.5) = 1` while `f(0.5) + f(0.5) = 0`.
///
/// Bounding a generic conversion on `RingHom` is therefore a correctness gate: truncation simply
/// has no impl, so it cannot be substituted for a widening by mistake.
pub trait RingHom: Hom {}
