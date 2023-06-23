/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
use crate::prelude::{Inferable, NumericalValue};
use crate::utils::math_utils::abs_num;

pub trait InferableReasoning<T>
    where
        T: Inferable,
{
    // Compiler generated methods using macros.
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_all_items(&self) -> Vec<&T>;
    // Default implementations.
    fn get_all_inferable(&self) -> Vec<&T> {
        self.get_all_items()
            .into_iter()
            .filter(|i| i.is_inferable())
            .collect()
    }
    fn get_all_inverse_inferable(&self) -> Vec<&T> {
        self.get_all_items()
            .into_iter()
            .filter(|i| i.is_inverse_inferable())
            .collect()
    }
    fn get_all_non_inferable(&self) -> Vec<&T> {
        // must be either or, but cannot be both b/c that would be undecidable hence non-inferable
        self.get_all_items()
            .into_iter()
            .filter(|i| i.is_inferable() && i.is_inverse_inferable())
            .collect()
    }
    /// returns true if all elements are inferable
    fn all_inferable(&self) -> bool {
        for element in self.get_all_items() {
            if !element.is_inferable() {
                return false;
            }
        }
        true
    }
    /// returns true if all elements are inverse inferable
    fn all_inverse_inferable(&self) -> bool {
        for element in self.get_all_items() {
            if !element.is_inverse_inferable() {
                return false;
            }
        }
        true
    }
    /// returns true if all elements are NON-inferable
    fn all_non_inferable(&self) -> bool {
        for element in self.get_all_items() {
            // must be either or, but cannot be both b/c that would be undecidable hence non-inferable
            if element.is_inverse_inferable() && element.is_inferable() {
                return true;
            }
        }
        false
    }
    /// The conjoint delta estimates the effect of those unobserverd conjoint factors.
    ///  conjoint_delta = abs(sum_cbservation/total))
    fn conjoint_delta(&self) -> NumericalValue {
        let one = 1.0;
        let total = self.len() as NumericalValue;
        let non_inferable = self.number_non_inferable();
        let cum_conjoint = total - non_inferable;

        abs_num(one - (cum_conjoint / total))
    }
    /// numbers inferable observations
    fn number_inferable(&self) -> NumericalValue {
        self.get_all_items()
            .into_iter()
            .filter(|i| i.is_inferable())
            .count() as NumericalValue
    }
    /// numbers inverse-inferable observations
    fn number_inverse_inferable(&self) -> NumericalValue {
        self.get_all_items()
            .into_iter()
            .filter(|i| i.is_inverse_inferable())
            .count() as NumericalValue
    }
    /// numbers non-inferable observations
    fn number_non_inferable(&self) -> NumericalValue {
        self.get_all_items()
            .into_iter()
            .filter(|i| i.is_inferable() && i.is_inverse_inferable())
            .count() as NumericalValue
    }
    /// percentage of observations that are inferable
    fn percent_inferable(&self) -> NumericalValue {
        let count = self.number_inferable();
        let total = self.len() as NumericalValue;
        (count / total) * (100 as NumericalValue)
    }
    /// percentage of observations that are inverse inferable
    fn percent_inverse_inferable(&self) -> NumericalValue {
        let count = self.number_inverse_inferable();
        let total = self.len() as NumericalValue;
        (count / total) * (100 as NumericalValue)
    }
    /// percentage of observations that are neither inferable nor inverse inferable
    fn percent_non_inferable(&self) -> NumericalValue {
        let count = self.number_non_inferable();
        let total = self.len() as NumericalValue;
        (count / total) * (100 as NumericalValue)
    }
}