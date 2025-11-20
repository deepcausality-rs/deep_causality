use crate::Quaternion;
use crate::ToPrimitive;
use crate::float::Float;

impl<F: Float> ToPrimitive for Quaternion<F> {
    /// Converts the scalar part (`w`) of the quaternion to an `isize`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_isize(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_isize(), None);
    /// ```
    fn to_isize(&self) -> Option<isize> {
        self.w.to_isize()
    }

    /// Converts the scalar part (`w`) of the quaternion to an `i8`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_i8(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_i8(), None);
    /// ```
    fn to_i8(&self) -> Option<i8> {
        self.w.to_i8()
    }

    /// Converts the scalar part (`w`) of the quaternion to an `i16`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_i16(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_i16(), None);
    /// ```
    fn to_i16(&self) -> Option<i16> {
        self.w.to_i16()
    }

    /// Converts the scalar part (`w`) of the quaternion to an `i32`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_i32(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_i32(), None);
    /// ```
    fn to_i32(&self) -> Option<i32> {
        self.w.to_i32()
    }

    /// Converts the scalar part (`w`) of the quaternion to an `i64`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_i64(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_i64(), None);
    /// ```
    fn to_i64(&self) -> Option<i64> {
        self.w.to_i64()
    }

    /// Converts the scalar part (`w`) of the quaternion to an `i128`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_i128(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_i128(), None);
    /// ```
    fn to_i128(&self) -> Option<i128> {
        self.w.to_i128()
    }

    /// Converts the scalar part (`w`) of the quaternion to a `usize`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_usize(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_usize(), None);
    /// ```
    fn to_usize(&self) -> Option<usize> {
        self.w.to_usize()
    }

    /// Converts the scalar part (`w`) of the quaternion to a `u8`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_u8(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_u8(), None);
    /// ```
    fn to_u8(&self) -> Option<u8> {
        self.w.to_u8()
    }

    /// Converts the scalar part (`w`) of the quaternion to a `u16`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_u16(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_u16(), None);
    /// ```
    fn to_u16(&self) -> Option<u16> {
        self.w.to_u16()
    }

    /// Converts the scalar part (`w`) of the quaternion to a `u32`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_u32(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_u32(), None);
    /// ```
    fn to_u32(&self) -> Option<u32> {
        self.w.to_u32()
    }

    /// Converts the scalar part (`w`) of the quaternion to a `u64`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_u64(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_u64(), None);
    /// ```
    fn to_u64(&self) -> Option<u64> {
        self.w.to_u64()
    }

    /// Converts the scalar part (`w`) of the quaternion to a `u128`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_u128(), Some(123));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert_eq!(q_nan.to_u128(), None);
    /// ```
    fn to_u128(&self) -> Option<u128> {
        self.w.to_u128()
    }

    /// Converts the scalar part (`w`) of the quaternion to an `f32`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_f32(), Some(123.45f32));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert!(q_nan.to_f32().unwrap().is_nan());
    /// ```
    fn to_f32(&self) -> Option<f32> {
        self.w.to_f32()
    }

    /// Converts the scalar part (`w`) of the quaternion to an `f64`.
    /// Returns `None` if the conversion is not possible (e.g., `NaN`, `Infinity`, or out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::ToPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// assert_eq!(q.to_f64(), Some(123.45f64));
    ///
    /// let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    /// assert!(q_nan.to_f64().unwrap().is_nan());
    /// ```
    fn to_f64(&self) -> Option<f64> {
        self.w.to_f64()
    }
}
