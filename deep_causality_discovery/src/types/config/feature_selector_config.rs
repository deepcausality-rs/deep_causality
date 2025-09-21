/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::config::mrmr_config::MrmrConfig;
use std::fmt;

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
