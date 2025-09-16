/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// A generic interface for casting between machine scalars with the
/// `as` operator, which admits narrowing and precision loss.
/// Implementers of this trait `AsPrimitive` should behave like a primitive
/// numeric type (e.g. a newtype around another primitive), and the
/// intended conversion must never fail.
///
/// # Examples
///
/// ```
/// use deep_causality_num::AsPrimitive;
/// let three: i32 = (3.14159265f32).as_();
/// assert_eq!(three, 3);
/// ```
///
/// # Safety
///
/// **In Rust versions before 1.45.0**, some uses of the `as` operator were not entirely safe.
/// In particular, it was undefined behavior if
/// a truncated floating point value could not fit in the target integer
/// type ([#10184](https://github.com/rust-lang/rust/issues/10184)).
///
/// ```ignore
/// # use deep_causality_num::AsPrimitive;
/// let x: u8 = (1.04E+17).as_(); // UB
/// ```
///
pub trait AsPrimitive<T>: 'static + Copy
where
    T: 'static + Copy,
{
    /// Convert a value to another, using the `as` operator.
    fn as_(self) -> T;
}

macro_rules! impl_as_primitive {
    (@ $T: ty => $(#[$cfg:meta])* impl $U: ty ) => {
        $(#[$cfg])*
        impl AsPrimitive<$U> for $T {
            #[inline] fn as_(self) -> $U { self as $U }
        }
    };
    (@ $T: ty => { $( $U: ty ),* } ) => {$(
        impl_as_primitive!(@ $T => impl $U);
    )*};
    ($T: ty => { $( $U: ty ),* } ) => {
        impl_as_primitive!(@ $T => { $( $U ),* });
        impl_as_primitive!(@ $T => { u8, u16, u32, u64, u128, usize });
        impl_as_primitive!(@ $T => { i8, i16, i32, i64, i128, isize });
    };
}

impl_as_primitive!(u8 => { char, f32, f64 });
impl_as_primitive!(i8 => { f32, f64 });
impl_as_primitive!(u16 => { f32, f64 });
impl_as_primitive!(i16 => { f32, f64 });
impl_as_primitive!(u32 => { f32, f64 });
impl_as_primitive!(i32 => { f32, f64 });
impl_as_primitive!(u64 => { f32, f64 });
impl_as_primitive!(i64 => { f32, f64 });
impl_as_primitive!(u128 => { f32, f64 });
impl_as_primitive!(i128 => { f32, f64 });
impl_as_primitive!(usize => { f32, f64 });
impl_as_primitive!(isize => { f32, f64 });
impl_as_primitive!(f32 => { f32, f64 });
impl_as_primitive!(f64 => { f32, f64 });
impl_as_primitive!(char => { char });
impl_as_primitive!(bool => {});
