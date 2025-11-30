/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::AddGroup;

/// An Abelian Group is a Group where the operation is Commutative.
///
/// Laws:
/// 1. Commutativity: a + b = b + a
///
/// Note: This is a Marker Trait. The compiler cannot verify commutativity,
/// so implementing this is a promise by the user.
pub trait AbelianGroup: AddGroup {}

// Automatic impls for standard types we know are Abelian
impl AbelianGroup for f32 {}
impl AbelianGroup for f64 {}
impl AbelianGroup for i8 {}
impl AbelianGroup for i16 {}
impl AbelianGroup for i32 {}
impl AbelianGroup for i64 {}
impl AbelianGroup for i128 {}
impl AbelianGroup for u8 {}
impl AbelianGroup for u16 {}
impl AbelianGroup for u32 {}
impl AbelianGroup for u64 {}
impl AbelianGroup for u128 {}

// Note: Complex, Dual, and MultiVector (Vector addition) are also Abelian.
// Impl this trait for them in their respective crates.
