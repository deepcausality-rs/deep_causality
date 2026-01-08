/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// ## Summary of the "Three Markers"
//
// | Type | `Distributive` | `Associative` | `Commutative` | Trait |
// | :--- | :---: | :---: | :---: | :--- |
// | **Real (`f64`)** | ✅ | ✅ | ✅ | `Field` |
// | **Complex** | ✅ | ✅ | ✅ | `Field` |
// | **Quaternion** | ✅ | ✅ | ❌ | `AssociativeRing` |
// | **Octonion** | ✅ | ❌ | ❌ | `DivisionAlgebra` |
// | **Matrix** | ✅ | ✅ | ❌ | `AssociativeRing` |
//

/// Marker trait: Promises that (a * b) * c == a * (b * c).
/// IMPLEMENT THIS for f64, f32, Float, Complex, Quaternion.
/// DO NOT IMPLEMENT for Octonion.
pub trait Associative {}
impl Associative for f32 {}
impl Associative for f64 {}
impl Associative for i8 {}
impl Associative for i16 {}
impl Associative for i32 {}
impl Associative for i64 {}
impl Associative for i128 {}
impl Associative for u8 {}
impl Associative for u16 {}
impl Associative for u32 {}
impl Associative for u64 {}
impl Associative for u128 {}
impl Associative for isize {}
impl Associative for usize {}

/// Marker trait: Promises that a * b == b * a.
/// IMPLEMENT THIS for f64, f32, Float, Complex.
/// DO NOT IMPLEMENT for Quaternion, Octonion.
pub trait Commutative {}

impl Commutative for f32 {}
impl Commutative for f64 {}
impl Commutative for i8 {}
impl Commutative for i16 {}
impl Commutative for i32 {}
impl Commutative for i64 {}
impl Commutative for i128 {}
impl Commutative for u8 {}
impl Commutative for u16 {}
impl Commutative for u32 {}
impl Commutative for u64 {}
impl Commutative for u128 {}
impl Commutative for isize {}
impl Commutative for usize {}

/// Marker trait: Promises that a * (b + c)  == (a * b) + (a * c).
pub trait Distributive {}
impl Distributive for f32 {}
impl Distributive for f64 {}
impl Distributive for i8 {}
impl Distributive for i16 {}
impl Distributive for i32 {}
impl Distributive for i64 {}
impl Distributive for i128 {}
impl Distributive for u8 {}
impl Distributive for u16 {}
impl Distributive for u32 {}
impl Distributive for u64 {}
impl Distributive for u128 {}
impl Distributive for isize {}
impl Distributive for usize {}
