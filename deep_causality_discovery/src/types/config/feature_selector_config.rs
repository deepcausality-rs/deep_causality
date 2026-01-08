/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::MrmrConfig;
use std::fmt;

/// An enum representing the configuration for a specific feature selection algorithm.
///
/// This allows the CDL pipeline to be configured for different feature selection
/// methods like MRMR.
#[derive(Debug, Clone)]
pub enum FeatureSelectorConfig {
    Mrmr(MrmrConfig),
}

impl fmt::Display for FeatureSelectorConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureSelectorConfig::Mrmr(c) => write!(f, "FeatureSelectorConfig::Mrmr({})", c),
        }
    }
}
