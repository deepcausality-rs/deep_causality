/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::SurdConfig;
use std::fmt;

/// An enum representing the configuration for a specific causal discovery algorithm.
///
/// This allows the CDL pipeline to be configured for different discovery methods
/// (e.g., SURD) by wrapping their specific configuration structs.
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
