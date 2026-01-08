/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt::{Display, Formatter};
use std::slice::Iter;

#[derive(Debug, Clone, PartialEq)]
pub struct MrmrResult {
    features: Vec<(usize, f64)>,
}

impl MrmrResult {
    /// Creates a new `MrmrResult` instance.
    pub fn new(features: Vec<(usize, f64)>) -> Self {
        Self { features }
    }

    /// Returns a reference to the selected features (index, score).
    pub fn features(&self) -> &[(usize, f64)] {
        &self.features
    }

    /// Returns an iterator over the selected features.
    pub fn iter(&self) -> Iter<'_, (usize, f64)> {
        self.features.iter()
    }

    /// Returns the number of selected features.
    pub fn len(&self) -> usize {
        self.features.len()
    }

    /// Returns true if no features were selected.
    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }
}

impl Display for MrmrResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "mRMR Selected Features:")?;
        writeln!(f, "-----------------------")?;
        writeln!(f, "{:<10} | {:<10}", "Index", "Score")?;
        writeln!(f, "{:-<10}-+-{:-<10}", "", "")?;

        for (idx, score) in &self.features {
            writeln!(f, "{:<10} | {:<10.4}", idx, score)?;
        }
        Ok(())
    }
}

impl IntoIterator for MrmrResult {
    type Item = (usize, f64);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.features.into_iter()
    }
}

impl<'a> IntoIterator for &'a MrmrResult {
    type Item = &'a (usize, f64);
    type IntoIter = std::slice::Iter<'a, (usize, f64)>;

    fn into_iter(self) -> Self::IntoIter {
        self.features.iter()
    }
}
