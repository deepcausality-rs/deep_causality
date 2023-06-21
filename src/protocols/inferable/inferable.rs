/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use std::cmp::Ordering;
use std::fmt::Debug;

use crate::prelude::{DescriptionValue, Identifiable, NumericalValue};
use crate::utils::math_utils::abs_num;

pub trait Inferable: Debug + Identifiable
{
    fn question(&self) -> DescriptionValue;
    fn observation(&self) -> NumericalValue;
    fn threshold(&self) -> NumericalValue;
    fn effect(&self) -> NumericalValue;
    fn target(&self) -> NumericalValue;

    fn conjoint_delta(&self) -> NumericalValue {
        abs_num((1.0) - self.observation())
    }

    fn is_inferable(&self) -> bool {
        if (self.observation().total_cmp(&self.threshold()) == Ordering::Greater)
            && approx_equal(self.effect(), self.target(), 4) {
            true
        } else {
            false
        }
    }

    fn is_inverse_inferable(&self) -> bool {
        if (self.observation().total_cmp(&self.threshold()) == Ordering::Less)
            && approx_equal(self.effect(), self.target(), 4) {
            true
        } else {
            false
        }
    }
}


// Because floats vary in precision, equality is not guaranteed.
// Therefore, this comparison checks for approximate equality up to a certain number
// of decimal places.
fn approx_equal(a: f64, b: f64, decimal_places: u8) -> bool {
    let factor = 10.0f64.powi(decimal_places as i32);
    let a = (a * factor).trunc();
    let b = (b * factor).trunc();
    a == b
}
