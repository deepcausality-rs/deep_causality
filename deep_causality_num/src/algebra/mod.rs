/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

///
/// The algebraic traits form a hierarchy:
///
/// Magma (Mul/Add operation)
///   ↓
/// Monoid (Magma + Identity)
///   ↓
/// Group (Monoid + Inverse)
///   ↓
/// AbelianGroup (Group + Commutative)
///   ↓
/// Ring (AbelianGroup + MulMonoid + Distributive)
///   ↓
/// CommutativeRing (Ring + Commutative Mul)
///   ↓
/// Field (CommutativeRing + Multiplicative Inverse)
///   ↓
/// RealField (Field + Ordering + Transcendentals)
///
pub mod algebra_assoc;
pub mod algebra_assoc_div;
pub mod algebra_base;
pub mod algebra_div;
pub mod algebra_properties;
pub mod field;
pub mod field_real;
pub mod group;
pub mod group_abelian;
pub mod group_add;
pub mod group_div;
pub mod group_mul;
pub mod magma;
pub mod module;
pub mod monoid;
pub mod ring;
pub mod ring_associative;
pub mod ring_com;
pub(crate) mod rotation;
