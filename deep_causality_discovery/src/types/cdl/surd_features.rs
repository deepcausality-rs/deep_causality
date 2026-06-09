/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{SurdFeatures, SurdResults};
use crate::{CDL, CdlBuilder, CdlEffect, CdlError, Precision};
use deep_causality_algorithms::surd::surd_states_cdl;

// After features are selected (SURD sub-pipeline)
impl<T: Precision> CDL<SurdFeatures<T>> {
    /// Runs SURD-states discovery using the max interaction order from the run
    /// config (no inline parameters).
    pub fn surd_discover(self) -> CdlEffect<CDL<SurdResults<T>>> {
        let config = self.state.config;
        let discovery_res = surd_states_cdl(&self.state.tensor, config.max_order());

        match discovery_res {
            Ok(surd_result) => CdlBuilder::pure(CDL {
                state: SurdResults {
                    surd_result,
                    selection_result: self.state.selection_result,
                    records_count: self.state.records_count,
                    config,
                },
            }),
            Err(e) => CdlEffect {
                inner: Err(CdlError::CausalDiscoveryError(e.into())),
                warnings: Default::default(),
            },
        }
    }
}

// Fluent stage method on the effect.
impl<T: Precision> CdlEffect<CDL<SurdFeatures<T>>> {
    /// See [`CDL::<SurdFeatures<T>>::surd_discover`].
    pub fn surd_discover(self) -> CdlEffect<CDL<SurdResults<T>>> {
        self.and_then(|cdl| cdl.surd_discover())
    }
}
