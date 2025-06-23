use crate::prelude::{AdjustableTime, TimeScale};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, Mul, Sub};

impl<T> AdjustableTime<T>
where
    T: Debug
        + Default
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Mul<T, Output = T>
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + PartialOrd,
{
    pub fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    pub fn time_unit(&self) -> T {
        self.time_unit
    }
}
