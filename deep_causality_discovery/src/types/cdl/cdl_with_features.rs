/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{WithCausalResults, WithFeatures};
use crate::{CDL, CausalDiscoveryError, CdlBuilder, CdlEffect, CdlError}; // Added CdlBuilder and Error type
use deep_causality_algorithms::causal_discovery::surd::SurdResult;
use deep_causality_tensor::CausalTensor;

// After features are selected
impl CDL<WithFeatures> {
    /// Runs a causal discovery algorithm on the feature-selected data.
    pub fn causal_discovery<F>(self, discovery_fn: F) -> CdlEffect<CDL<WithCausalResults>>
    where
        F: FnOnce(&CausalTensor<Option<f64>>) -> Result<SurdResult<f64>, CausalDiscoveryError>,
    {
        let discovery_res = discovery_fn(&self.state.tensor);

        match discovery_res {
            Ok(results) => CdlBuilder::pure(CDL {
                state: WithCausalResults {
                    surd_result: results,
                    selection_result: self.state.selection_result,
                    records_count: self.state.records_count,
                },
                config: self.config,
            }),
            // Map CausalTensorError/AlgoError to CdlError
            Err(e) => CdlEffect {
                inner: Err(CdlError::CausalDiscoveryError(e)),
                warnings: Default::default(),
            },
        }
    }
}
