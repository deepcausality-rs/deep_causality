/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Ring;

/// An Associative Ring is a Ring where multiplication is associative.
///
/// Since the `MulMonoid` trait (which `Ring` transitively requires) already
/// enforces associativity of multiplication, this trait serves primarily
/// as a semantic marker for types that explicitly represent an associative ring.
///
/// Laws (inherited from Ring and MulMonoid):
/// 1. Addition forms an Abelian Group.
/// 2. Multiplication forms an Associative Monoid.
/// 3. Distributivity holds (multiplication distributes over addition).
pub trait AssociativeRing: Ring {}

// Blanket Implementation for all types that implement Ring
impl<T> AssociativeRing for T where T: Ring {}
