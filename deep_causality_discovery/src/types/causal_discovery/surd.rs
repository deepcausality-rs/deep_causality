/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::CausalDiscoveryError;
use crate::traits::causal_discovery::CausalDiscovery;
use crate::types::config::CausalDiscoveryConfig;
use deep_causality_algorithms::surd::{SurdResult, surd_states};
use deep_causality_tensor::CausalTensor;

/// A concrete implementation of the `CausalDiscovery` trait using the SURD algorithm.
///
/// This struct acts as a bridge between the CDL pipeline and the `surd_states`
/// algorithms from the `deep_causality_algorithms` crate.
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
