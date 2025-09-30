/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{WithCausalResults, WithFeatures};
use crate::{CDL, CausalDiscovery, CdlError};

// After features are selected
impl CDL<WithFeatures> {
    /// Runs a causal discovery algorithm on the feature-selected data.
    ///
    /// # Arguments
    /// * `discovery` - An implementation of `CausalDiscovery` (e.g., `SurdCausalDiscovery`).
    ///
    /// # Returns
    /// A `CDL` instance in the `WithCausalResults` state, or a `CdlError` if discovery fails.
    pub fn causal_discovery<D>(self, discovery: D) -> Result<CDL<WithCausalResults>, CdlError>
    where
        D: CausalDiscovery,
    {
        let discovery_config = self
            .config
            .causal_discovery_config()
            .as_ref()
            .ok_or(CdlError::MissingCausalDiscoveryConfig)?;

        let results = discovery.discover(self.state.0, discovery_config)?;
        Ok(CDL {
            state: WithCausalResults(results),
            config: self.config,
        })
    }
}
