/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalDiscovery;
use crate::{CausalDiscoveryConfig, CausalDiscoveryError};
use deep_causality_algorithms::surd::{SurdResult, surd_states_cdl};
use deep_causality_tensor::CausalTensor;

/// A concrete implementation of the `CausalDiscovery` trait using the SURD algorithm.
///
/// This struct acts as a bridge between the CDL pipeline and the `surd_states`
/// algorithms from the `deep_causality_algorithms` crate.
pub struct SurdCausalDiscovery;

impl CausalDiscovery for SurdCausalDiscovery {
    fn discover(
        &self,
        tensor: CausalTensor<Option<f64>>,
        config: &CausalDiscoveryConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError> {
        let CausalDiscoveryConfig::Surd(surd_config) = config;
        Self::discover_res(&tensor, surd_config)
    }
}

impl SurdCausalDiscovery {
    pub fn discover_res(
        tensor: &CausalTensor<Option<f64>>,
        config: &crate::SurdConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError> {
        Ok(surd_states_cdl(tensor, config.max_order())?)
    }
}
