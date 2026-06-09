/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{BrcdLoaded, BrcdResults};
use crate::{CDL, CausalDiscoveryError, CdlBuilder, CdlEffect, CdlError, Precision};
use deep_causality_algorithms::brcd::brcd_run;
use deep_causality_num::ToPrimitive;

// After the BRCD input bundle is loaded (BRCD sub-pipeline)
impl<T: Precision + ToPrimitive> CDL<BrcdLoaded<T>> {
    /// Runs BRCD using the configuration carried in the loaded bundle.
    ///
    /// Calls [`brcd_run`] with the two datasets and the optional CPDAG. When the
    /// CPDAG is `None`, `brcd_run` learns it from the normal data via BOSS. A
    /// BRCD failure surfaces as a `CdlError` wrapping
    /// [`CausalDiscoveryError::Brcd`].
    pub fn brcd_discover(self) -> CdlEffect<CDL<BrcdResults<T>>> {
        let dataset_path = self.state.dataset_path;
        let input = &self.state.input;
        let records_count = input.normal().shape()[0] + input.anomalous().shape()[0];

        let result = brcd_run(
            input.normal(),
            input.anomalous(),
            input.cpdag(),
            input.brcd_config(),
        );

        match result {
            Ok(brcd_result) => CdlBuilder::pure(CDL {
                state: BrcdResults {
                    brcd_result,
                    records_count,
                    dataset_path,
                },
            }),
            Err(e) => CdlEffect {
                inner: Err(CdlError::from(CausalDiscoveryError::from(e))),
                warnings: Default::default(),
            },
        }
    }
}

// Fluent stage method on the effect.
impl<T: Precision + ToPrimitive> CdlEffect<CDL<BrcdLoaded<T>>> {
    /// See [`CDL::<BrcdLoaded<T>>::brcd_discover`].
    pub fn brcd_discover(self) -> CdlEffect<CDL<BrcdResults<T>>> {
        self.and_then(|cdl| cdl.brcd_discover())
    }
}
