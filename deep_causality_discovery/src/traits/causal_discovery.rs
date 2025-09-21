/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalDiscoveryConfig;
use crate::errors::CausalDiscoveryError;
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_tensor::CausalTensor;

pub trait CausalDiscovery {
    fn discover(
        &self,
        tensor: CausalTensor<f64>,
        config: &CausalDiscoveryConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError>;
}
