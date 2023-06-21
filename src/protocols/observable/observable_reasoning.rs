/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use crate::prelude::{NumericalValue, Observable};

pub trait ObservableCollection<T>
    where
        T: Observable,
{
    // Compiler generated methods using macros.
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_all_items(&self) -> Vec<&T>;

    fn number_observation(
        &self,
        target_threshold: NumericalValue,
        target_effect: NumericalValue,
    ) -> NumericalValue
    {
        self.get_all_items()
            .iter()
            .filter(|o| o.effect_observed(target_threshold, target_effect))
            .count() as NumericalValue
    }
    fn number_non_observation(
        &self,
        target_threshold: NumericalValue,
        target_effect: NumericalValue,
    ) -> NumericalValue
    {
        let num_obs = self.number_observation(target_threshold, target_effect);
        let total = self.len() as NumericalValue;
        total - num_obs
    }


    fn percent_observation(
        &self,
        target_threshold: NumericalValue,
        target_effect: NumericalValue,
    ) -> NumericalValue
    {
        let total = self.len() as NumericalValue;
        let number = self.number_observation(target_threshold, target_effect);
        number / total // * (100 as NumericalValue)
    }
    fn percent_non_observation(
        &self,
        target_threshold: NumericalValue,
        target_effect: NumericalValue,
    ) -> NumericalValue
    {
        let perc_obs = self.percent_observation(target_threshold, target_effect);
        1.0 - perc_obs
    }
}