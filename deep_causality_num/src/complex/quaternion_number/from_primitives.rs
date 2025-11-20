use crate::FromPrimitive;
use crate::complex::quaternion_number::Quaternion;
use crate::float::Float;

impl<F: Float> FromPrimitive for Quaternion<F> {
    fn from_isize(n: isize) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_i8(n: i8) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_i16(n: i16) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_i32(n: i32) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_i64(n: i64) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_i128(n: i128) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_usize(n: usize) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_u8(n: u8) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_u16(n: u16) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_u32(n: u32) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_u128(n: u128) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_f32(n: f32) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    fn from_f64(n: f64) -> Option<Self> {
        Some(Quaternion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }
}
