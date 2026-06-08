/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::BrcdConfig;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

/// The fully-prepared input for a BRCD discovery run.
///
/// Produced by [`crate::BrcdDataLoader`] and consumed by the BRCD lineage's
/// `load_brcd_input`. It bundles the two aligned datasets, the optional CPDAG,
/// and the reused algorithm config into one value so the discovery stage has
/// everything it needs without further loading. When `cpdag` is `None`, BRCD
/// learns the CPDAG from the normal data via BOSS.
#[derive(Debug)]
pub struct BrcdInput<T> {
    normal: CausalTensor<T>,
    anomalous: CausalTensor<T>,
    cpdag: Option<MixedGraph<()>>,
    brcd_config: BrcdConfig<T>,
}

impl<T> BrcdInput<T> {
    /// Creates a new BRCD input bundle.
    pub fn new(
        normal: CausalTensor<T>,
        anomalous: CausalTensor<T>,
        cpdag: Option<MixedGraph<()>>,
        brcd_config: BrcdConfig<T>,
    ) -> Self {
        Self {
            normal,
            anomalous,
            cpdag,
            brcd_config,
        }
    }
}

// --- Getters ----------------------------------------------------------------
impl<T> BrcdInput<T> {
    /// The observational ("normal") dataset.
    pub fn normal(&self) -> &CausalTensor<T> {
        &self.normal
    }

    /// The failure ("anomalous") dataset.
    pub fn anomalous(&self) -> &CausalTensor<T> {
        &self.anomalous
    }

    /// The optional CPDAG over the variables. `None` defers structure learning
    /// to BOSS inside the BRCD driver.
    pub fn cpdag(&self) -> Option<&MixedGraph<()>> {
        self.cpdag.as_ref()
    }

    /// The reused algorithm configuration.
    pub fn brcd_config(&self) -> &BrcdConfig<T> {
        &self.brcd_config
    }
}
