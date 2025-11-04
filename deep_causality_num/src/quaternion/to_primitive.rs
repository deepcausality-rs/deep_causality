use crate::ToPrimitive;
use crate::float::Float;
use crate::quaternion::Quaternion;

impl<F: Float> ToPrimitive for Quaternion<F> {
    fn to_isize(&self) -> Option<isize> {
        self.w.to_isize()
    }

    fn to_i8(&self) -> Option<i8> {
        self.w.to_i8()
    }

    fn to_i16(&self) -> Option<i16> {
        self.w.to_i16()
    }

    fn to_i32(&self) -> Option<i32> {
        self.w.to_i32()
    }

    fn to_i64(&self) -> Option<i64> {
        self.w.to_i64()
    }

    fn to_i128(&self) -> Option<i128> {
        self.w.to_i128()
    }

    fn to_usize(&self) -> Option<usize> {
        self.w.to_usize()
    }

    fn to_u8(&self) -> Option<u8> {
        self.w.to_u8()
    }

    fn to_u16(&self) -> Option<u16> {
        self.w.to_u16()
    }

    fn to_u32(&self) -> Option<u32> {
        self.w.to_u32()
    }

    fn to_u64(&self) -> Option<u64> {
        self.w.to_u64()
    }

    fn to_u128(&self) -> Option<u128> {
        self.w.to_u128()
    }

    fn to_f32(&self) -> Option<f32> {
        self.w.to_f32()
    }

    fn to_f64(&self) -> Option<f64> {
        self.w.to_f64()
    }
}
