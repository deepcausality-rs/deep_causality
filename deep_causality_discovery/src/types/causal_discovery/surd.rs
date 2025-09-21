/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::CausalDiscoveryError;
use crate::traits::causal_discovery::CausalDiscovery;
use crate::types::config::CausalDiscoveryConfig;
use deep_causality_algorithms::surd::{SurdResult, surd_states};
use deep_causality_tensor::CausalTensor;

pub struct SurdCausalDiscovery;

impl CausalDiscovery for SurdCausalDiscovery {
    fn discover(
        &self,
        tensor: CausalTensor<f64>,
        config: &CausalDiscoveryConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError> {
        let CausalDiscoveryConfig::Surd(surd_config) = config;
        Ok(surd_states(&tensor, surd_config.max_order())?)
    }
}
