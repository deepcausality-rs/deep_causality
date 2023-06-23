/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use std::fmt::Debug;

use crate::prelude::{Identifiable, NumericalValue};

pub trait Observable: Debug + Identifiable {
    fn observation(&self) -> NumericalValue;
    fn observed_effect(&self) -> NumericalValue;

    fn effect_observed(
        &self,
        target_threshold: NumericalValue,
        target_effect: NumericalValue,
    ) -> bool {
        (self.observation() >= target_threshold) && (self.observed_effect() == target_effect)
    }
}
