// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

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

pub trait ObservableReasoning<T>
    where
        T: Observable,
{
    // Compiler generated methods using macros.
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_all_items(&self) -> Vec<&T>;

    // Default implementations.

    fn number_observation(
        &self,
        target_threshold: NumericalValue,
        target_effect: NumericalValue,
    )
        -> NumericalValue
    {
        self.get_all_items().iter().filter(|o| o.effect_observed(target_threshold, target_effect)).count() as NumericalValue
    }

    fn number_non_observation(
        &self,
        target_threshold: NumericalValue,
        target_effect: NumericalValue,
    )
        -> NumericalValue
    {
        self.len() as NumericalValue - self.number_observation(target_threshold, target_effect)
    }

    fn percent_observation(
        &self,
        target_threshold: NumericalValue,
        target_effect: NumericalValue,
    )
        -> NumericalValue
    {
        self.number_observation(target_threshold, target_effect) / self.len() as NumericalValue // * (100 as NumericalValue)
    }

    fn percent_non_observation(
        &self,
        target_threshold: NumericalValue,
        target_effect: NumericalValue,
    )
        -> NumericalValue
    {
        1.0 - self.percent_observation(target_threshold, target_effect)
    }
}
