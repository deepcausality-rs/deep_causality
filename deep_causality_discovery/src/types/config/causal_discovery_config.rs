/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::config::surd_config::SurdConfig;
use std::fmt;

#[derive(Debug, Clone)]
pub enum CausalDiscoveryConfig {
    Surd(SurdConfig),
}

impl fmt::Display for CausalDiscoveryConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CausalDiscoveryConfig::Surd(c) => write!(f, "CausalDiscoveryConfig::Surd({})", c),
        }
    }
}
