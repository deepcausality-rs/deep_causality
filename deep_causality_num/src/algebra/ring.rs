/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AbelianGroup, MulMonoid};

///
/// A Ring is a set equipped with Addition (+) and Multiplication (*).
///
/// ## Structure:
/// 1. Addition forms an Abelian Group (Commutative, Inverse, Identity 0).
/// 2. Multiplication forms a Monoid (Associative, Identity 1).
/// 3. Multiplication distributes over Addition: a*(b+c) = a*b + a*c.
///
/// ## Requirements:
/// 1. Addition is an Abelian Group.
/// 2. Multiplication is a Monoid (Associative).
/// 3. Distributivity holds (implied).
pub trait Ring: AbelianGroup + MulMonoid {
    // No new methods needed.
    // It just guarantees you can use +, -, *, 0, 1.
}

// Blanket Implementation for all types that implement Add, Sub, Mul, MulAssign, and One
impl<T> Ring for T where T: AbelianGroup + MulMonoid {}
